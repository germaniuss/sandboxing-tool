use std::fs::{File, OpenOptions};
use std::os::fd::OwnedFd;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, io};

use clap::Parser;
use conf::Conf;
use figment::Figment;
use figment::providers::{Format, Toml};
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use serde::{Deserialize, Deserializer};

const DEFAULT_PATH: &str = "profile.toml";

#[derive(Debug, PartialEq, Eq)]
enum ParseSizeError {
    InvalidFormat,
    InvalidNumber,
    Overflow,
}

impl fmt::Display for ParseSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseSizeError::InvalidFormat => write!(
                f,
                "Invalid size format. Expected numbers followed by units like 'MiB' or 'GiB'"
            ),
            ParseSizeError::InvalidNumber => {
                write!(f, "Could not parse the numeric portion of the size")
            }
            ParseSizeError::Overflow => write!(
                f,
                "The specified size is too large and caused an integer overflow"
            ),
        }
    }
}

fn parse_iec_size(input: &str) -> Result<u64, ParseSizeError> {
    let trimmed = input.trim();

    let digit_end = trimmed
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(trimmed.len());

    let (num_str, unit_str) = trimmed.split_at(digit_end);

    if num_str.is_empty() {
        return Err(ParseSizeError::InvalidFormat);
    }

    let base_num = u64::from_str(num_str).map_err(|_| ParseSizeError::InvalidNumber)?;

    let multiplier: u64 = match unit_str.to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" | "kib" => 1024,
        "m" | "mb" | "mib" => 1024_u64.pow(2),
        "g" | "gb" | "gib" => 1024_u64.pow(3),
        "t" | "tb" | "tib" => 1024_u64.pow(4),
        _ => return Err(ParseSizeError::InvalidFormat),
    };

    base_num
        .checked_mul(multiplier)
        .ok_or(ParseSizeError::Overflow)
}

fn deserialize_optional_iec_size<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;

    match opt {
        Some(s) => parse_iec_size(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

use std::num::ParseFloatError;

#[derive(Debug)]
enum CpuLimitError {
    ParseError(ParseFloatError),
    NegativeValue(f64),
    NotANumber,
    InfiniteValue,
    TooLarge,
}

impl fmt::Display for CpuLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "Invalid CPU string format: {}", e),
            Self::NegativeValue(v) => write!(f, "CPU value cannot be negative: {}", v),
            Self::NotANumber => write!(f, "CPU value cannot be NaN"),
            Self::InfiniteValue => write!(f, "CPU value cannot be infinite"),
            Self::TooLarge => write!(f, "CPU value is too large to fit in a u64"),
        }
    }
}

fn cpu_float_to_microseconds(cpu: f64) -> Result<u64, CpuLimitError> {
    if cpu.is_nan() {
        return Err(CpuLimitError::NotANumber);
    }
    if cpu.is_infinite() {
        return Err(CpuLimitError::InfiniteValue);
    }
    if cpu < 0.0 {
        return Err(CpuLimitError::NegativeValue(cpu));
    }

    let microsecs = cpu * 1_000_000.0;

    if microsecs > u64::MAX as f64 {
        return Err(CpuLimitError::TooLarge);
    }

    Ok(microsecs.round() as u64)
}

fn parse_cpu_string_to_microseconds(s: &str) -> Result<u64, CpuLimitError> {
    let cpu: f64 = s.parse().map_err(CpuLimitError::ParseError)?;
    cpu_float_to_microseconds(cpu)
}

fn deserialize_cpu_string_to_microseconds<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;

    match opt {
        Some(s) => cpu_float_to_microseconds(s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

fn parse_path_mapping(s: &str) -> Result<(PathBuf, PathBuf, bool), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err("Expected format: source:target[:rw|ro]".into());
    }
    let source = PathBuf::from(parts[0]);
    let target = PathBuf::from(parts[1]);
    // we should read a comma separated list of options
    // like docker's --volume
    let readonly = match parts.get(2) {
        None => true,
        Some(&"ro") => true,
        Some(&"rw") => false,
        Some(other) => return Err(format!("Invalid flag: {}", other)),
    };
    Ok((source, target, readonly))
}

const fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct BindCommon {
    source: PathBuf,
    target: PathBuf,
}

#[derive(Deserialize)]
pub struct Bind {
    #[serde(flatten)]
    common: BindCommon,
    #[serde(default = "default_true")]
    readonly: bool,
}

#[derive(Deserialize)]
pub struct Device {
    #[serde(flatten)]
    common: BindCommon,
    #[serde(default = "default_true")]
    readonly: bool,
}

impl Into<sandbox::operations::Bind> for &Bind {
    fn into(self) -> sandbox::operations::Bind {
        sandbox::operations::Bind {
            source: self.common.source.clone(),
            target: self.common.target.clone(),
            readonly: self.readonly,
            device: false,
            recursive: true,
        }
    }
}

impl Into<sandbox::operations::Bind> for &Device {
    fn into(self) -> sandbox::operations::Bind {
        sandbox::operations::Bind {
            source: self.common.source.clone(),
            target: self.common.target.clone(),
            readonly: self.readonly,
            device: true,
            recursive: true,
        }
    }
}

#[derive(Deserialize)]
pub struct Symlink {
    source: PathBuf,
    target: PathBuf,
}

impl Into<sandbox::operations::Symlink> for &Symlink {
    fn into(self) -> sandbox::operations::Symlink {
        sandbox::operations::Symlink {
            source: self.source.clone(),
            target: self.target.clone(),
        }
    }
}

impl FromStr for Bind {
    type Err = String;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err("default implementation because conf-rs is bad".into())
    }
}

impl FromStr for Device {
    type Err = String;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err("default implementation because conf-rs is bad".into())
    }
}

impl FromStr for Symlink {
    type Err = String;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err("default implementation because conf-rs is bad".into())
    }
}

fn parse_device(s: &str) -> Result<Device, String> {
    let (source, target, readonly) = parse_path_mapping(s)?;
    Ok(Device {
        common: BindCommon { source, target },
        readonly,
    })
}

fn parse_bind_mount(s: &str) -> Result<Bind, String> {
    let (source, target, readonly) = parse_path_mapping(s)?;
    Ok(Bind {
        common: BindCommon { source, target },
        readonly,
    })
}

fn parse_symlink(s: &str) -> Result<Symlink, String> {
    let (source, target, _) = parse_path_mapping(s)?;
    Ok(Symlink { source, target })
}

pub enum Stdio {
    Null,
    Inherit,
    File {
        file: File,
        path: PathBuf,
        created: bool,
        fifo: Option<OwnedFd>,
    },
}

fn parse_stdout_stderr(s: &str) -> io::Result<Stdio> {
    if s.to_lowercase() == "inherit" {
        Ok(Stdio::Inherit)
    } else if s.to_lowercase() == "null" || s.to_lowercase() == "none" {
        Ok(Stdio::Null)
    } else {
        let path = Into::<PathBuf>::into(s);
        let (file, created) = if path.exists() {
            let meta = std::fs::metadata(&path)?;
            if meta.is_dir() || meta.is_symlink() {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            (
                OpenOptions::new().write(true).truncate(true).open(&path)?,
                false,
            )
        } else {
            (
                OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create_new(true)
                    .open(&path)?,
                true,
            )
        };
        Ok(Stdio::File {
            file,
            path,
            created,
            fifo: None,
        })
    }
}

fn parse_stdin(s: &str) -> io::Result<Stdio> {
    if s.to_lowercase() == "inherit" {
        Ok(Stdio::Inherit)
    } else if s.to_lowercase() == "null" || s.to_lowercase() == "none" {
        Ok(Stdio::Null)
    } else {
        let path = Into::<PathBuf>::into(s);
        let created = if path.exists() {
            let meta = std::fs::metadata(&path)?;
            if meta.is_dir() || meta.is_symlink() {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            false
        } else {
            mkfifo(
                &path,
                Mode::S_IRGRP
                    | Mode::S_IRUSR
                    | Mode::S_IROTH
                    | Mode::S_IWGRP
                    | Mode::S_IWUSR
                    | Mode::S_IWOTH,
            )?;
            true
        };
        let meta = std::fs::metadata(&path)?;
        let fifo = if meta.file_type().is_fifo() {
            let fifo = open(&path, OFlag::O_NONBLOCK | OFlag::O_RDWR, Mode::empty())?;
            Some(fifo)
        } else {
            None
        };

        let file = OpenOptions::new().read(true).open(&path)?;

        Ok(Stdio::File {
            file,
            path,
            created,
            fifo,
        })
    }
}

impl TryInto<sandbox::process::Stdio> for Stdio {
    type Error = io::Error;
    fn try_into(self) -> io::Result<sandbox::process::Stdio> {
        match self {
            Stdio::Null => Ok(sandbox::process::Stdio::null()),
            Stdio::Inherit => Ok(sandbox::process::Stdio::inherit()),
            Stdio::File { file, .. } => Ok(sandbox::process::Stdio::from(file)),
        }
    }
}

impl FromStr for Stdio {
    type Err = String;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err("default implementation because conf-rs is bad".into())
    }
}

fn deserialize_stdin<'de, D>(deserializer: D) -> Result<Option<Stdio>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;

    match opt {
        Some(s) => parse_stdin(s.as_str())
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

fn deserialize_stdout_stderr<'de, D>(deserializer: D) -> Result<Option<Stdio>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;

    match opt {
        Some(s) => parse_stdout_stderr(s.as_str())
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

#[derive(Conf)]
#[conf(serde)]
pub struct Resources {
    #[conf(long, value_parser = parse_iec_size, serde(deserialize_with = "deserialize_optional_iec_size"))]
    pub memory: Option<u64>,
    #[conf(long, value_parser = parse_cpu_string_to_microseconds, serde(deserialize_with = "deserialize_cpu_string_to_microseconds"))]
    pub cpus: Option<u64>,
}

#[derive(Conf)]
#[conf(serde)]
pub struct Operations {
    #[conf(repeat, long = "bind", value_parser = parse_bind_mount)]
    pub binds: Vec<Bind>,
    #[conf(repeat, long = "device", value_parser = parse_device)]
    pub devices: Vec<Device>,
    #[conf(repeat, long = "symlink", value_parser = parse_symlink)]
    pub symlinks: Vec<Symlink>,
}

#[derive(Conf)]
#[conf(serde)]
pub struct StdioOptions {
    #[conf(long, value_parser = parse_stdin, serde(deserialize_with = "deserialize_stdin"))]
    pub stdin: Option<Stdio>,
    #[conf(long, value_parser = parse_stdout_stderr,  serde(deserialize_with = "deserialize_stdout_stderr"))]
    pub stdout: Option<Stdio>,
    #[conf(long, value_parser = parse_stdout_stderr,  serde(deserialize_with = "deserialize_stdout_stderr"))]
    pub stderr: Option<Stdio>,
}

#[derive(Conf)]
#[conf(serde, version)]
pub struct Profile {
    #[conf(repeat, short = 'f', long = "profile")]
    pub paths: Vec<PathBuf>,
    #[conf(flatten)]
    pub resources: Option<Resources>,
    #[conf(flatten, serde(flatten))]
    pub operations: Operations,
    #[conf(flatten)]
    pub stdio: StdioOptions,
    #[conf(short, long)]
    pub timeout: Option<u64>,
    #[conf(short, long)]
    pub wait: bool,
    #[conf(pos, serde(skip))]
    pub command: String,
    #[conf(repeat, pos, serde(skip))]
    pub args: Vec<String>,
}

#[derive(clap::Parser, Debug)]
#[command(
    ignore_errors = true,
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(short = 'f', long = "profile")]
    pub paths: Vec<PathBuf>,
}

pub fn profile() -> Result<Profile, figment::Error> {
    // Parse profile paths from CLI is non were provided use DEFAULT_PATH if exists,
    // if not then just use arguments from the terminal
    let mut cli = Cli::parse();

    if cli.paths.is_empty() && Path::new(DEFAULT_PATH).exists() {
        cli.paths.push(DEFAULT_PATH.into());
    }

    if cli.paths.is_empty() {
        return Ok(Profile::parse());
    }

    let paths_str = cli
        .paths
        .iter()
        .map(|p| p.to_string_lossy())
        .collect::<Vec<_>>()
        .join(":");

    let content: toml::Value = cli
        .paths
        .iter()
        .fold(Figment::new(), |f, path| f.merge(Toml::file(path)))
        .extract()?;

    let mut profile = Profile::conf_builder().doc(paths_str, content).parse();

    profile.paths = cli.paths;

    Ok(profile)
}
