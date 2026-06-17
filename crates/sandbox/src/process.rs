use std::{
    ffi::{OsStr, OsString},
    fmt,
    fs::{File, OpenOptions},
    io,
    sync::OnceLock,
};

use nix::{
    sys::{
        signal::{Signal, kill},
        wait::{WaitStatus, waitpid},
    },
    unistd::Pid,
};

use crate::sandbox;

static NULL_FILE: OnceLock<File> = OnceLock::new();

pub fn null_file() -> io::Result<&'static File> {
    if let Some(file) = NULL_FILE.get() {
        return Ok(file);
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")?;
    let _ = NULL_FILE.set(file);
    Ok(NULL_FILE.get().unwrap())
}

#[derive(Debug)]
pub enum Stdio {
    Null,
    Inherit,
    Piped,
    File(File),
}

impl Stdio {
    pub fn null() -> Self {
        Stdio::Null
    }
    pub fn inherit() -> Self {
        Stdio::Inherit
    }
    pub fn piped() -> Self {
        Stdio::Piped
    }
}

impl TryFrom<&Stdio> for std::process::Stdio {
    type Error = io::Error;

    fn try_from(stdio: &Stdio) -> io::Result<Self> {
        match stdio {
            Stdio::Null => {
                let null_file = null_file()?.try_clone()?;
                Ok(std::process::Stdio::from(null_file))
            }
            Stdio::Inherit => Ok(std::process::Stdio::inherit()),
            Stdio::Piped => Ok(std::process::Stdio::piped()),
            Stdio::File(file) => Ok(std::process::Stdio::from(file.try_clone()?)),
        }
    }
}

impl TryFrom<Stdio> for std::process::Stdio {
    type Error = io::Error;

    fn try_from(stdio: Stdio) -> Result<Self, Self::Error> {
        std::process::Stdio::try_from(&stdio)
    }
}

impl From<File> for Stdio {
    fn from(file: File) -> Self {
        Stdio::File(file)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    Exited(i32),
    Signaled(Signal),
}

impl ExitStatus {
    pub fn success(&self) -> bool {
        matches!(self, Self::Exited(0))
    }

    pub fn code(&self) -> Option<i32> {
        match self {
            Self::Exited(code) => Some(*code),
            Self::Signaled(_) => None,
        }
    }

    pub fn signal(&self) -> Option<Signal> {
        match self {
            Self::Exited(_) => None,
            Self::Signaled(sig) => Some(*sig),
        }
    }

    pub fn from_wait_status(status: WaitStatus) -> Option<Self> {
        match status {
            WaitStatus::Exited(_, code) => Some(Self::Exited(code)),
            WaitStatus::Signaled(_, signal, _) => Some(Self::Signaled(signal)),
            WaitStatus::StillAlive
            | WaitStatus::Stopped(_, _)
            | WaitStatus::PtraceEvent(_, _, _)
            | WaitStatus::PtraceSyscall(_)
            | WaitStatus::Continued(_) => None,
        }
    }
}

impl TryFrom<WaitStatus> for ExitStatus {
    type Error = io::Error;
    fn try_from(status: WaitStatus) -> io::Result<Self> {
        match status {
            WaitStatus::Exited(_, code) => Ok(Self::Exited(code)),
            WaitStatus::Signaled(_, signal, _) => Ok(Self::Signaled(signal)),
            WaitStatus::StillAlive => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process is still alive",
            )),
            WaitStatus::Stopped(_, signal) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("process stopped by signal {signal:?}"),
            )),
            WaitStatus::Continued(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process has been continued",
            )),
            WaitStatus::PtraceEvent(_, signal, event) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("ptrace event {event} ({signal:?})"),
            )),
            WaitStatus::PtraceSyscall(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ptrace syscall stop",
            )),
        }
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExitStatus::Exited(code) => {
                write!(f, "exit status: {}", code)
            }
            ExitStatus::Signaled(signal) => {
                write!(f, "signal: {}", *signal as i32)
            }
        }
    }
}

pub struct Child {
    pub(crate) pid: Pid,
    pub(crate) kill_on_drop: bool,
    pub(crate) reap_on_drop: bool,
}

impl Child {
    pub fn getpid(&self) -> &Pid {
        &self.pid
    }

    fn new(command: &mut Command) -> io::Result<Child> {
        let kill_on_drop = command.kill_on_drop;
        let reap_on_drop = command.reap_on_drop;
        let pid = sandbox::Sandbox::spawn(command)?;
        Ok(Child {
            pid: pid,
            kill_on_drop: kill_on_drop,
            reap_on_drop: reap_on_drop,
        })
    }

    pub fn wait(&self) -> io::Result<ExitStatus> {
        let status = waitpid(self.pid, None)?;
        Ok(ExitStatus::try_from(status)?)
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        if self.kill_on_drop {
            let _ = kill(self.pid, Signal::SIGKILL);
        }
        if self.reap_on_drop {
            let _ = waitpid(self.pid, None);
        }
    }
}

pub struct Command<'a> {
    pub(crate) inner: &'a mut sandbox::Sandbox,
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,

    pub(crate) stdin: Stdio,
    pub(crate) stdout: Stdio,
    pub(crate) stderr: Stdio,

    kill_on_drop: bool,
    reap_on_drop: bool,

    pub(crate) die_with_parent_thread: bool,

    pub(crate) reparent_to_pid1_in_ns: bool,
}

impl<'a> Command<'a> {
    pub(crate) fn new<S: AsRef<OsStr>>(inner: &'a mut sandbox::Sandbox, program: S) -> Command<'a> {
        Command {
            inner: inner,
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            stdin: Stdio::null(),
            stdout: Stdio::null(),
            stderr: Stdio::null(),
            kill_on_drop: false,
            reap_on_drop: false,
            die_with_parent_thread: false,
            reparent_to_pid1_in_ns: false,
        }
    }

    pub fn stdin(&mut self, stream: Stdio) -> &mut Self {
        self.stdin = stream;
        self
    }

    pub fn stdout(&mut self, stream: Stdio) -> &mut Self {
        self.stdout = stream;
        self
    }

    pub fn stderr(&mut self, stream: Stdio) -> &mut Self {
        self.stderr = stream;
        self
    }

    pub fn die_with_parent_thread(&mut self, value: bool) -> &mut Self {
        self.die_with_parent_thread = value;
        self
    }

    pub fn reparent_to_pid1_in_ns(&mut self, value: bool) -> &mut Self {
        self.reparent_to_pid1_in_ns = value;
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_owned()));
        self
    }

    pub fn kill_on_drop(&mut self, value: bool) -> &mut Self {
        self.kill_on_drop = value;
        self
    }

    pub fn reap_on_drop(&mut self, value: bool) -> &mut Self {
        self.reap_on_drop = value;
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        Child::new(self)
    }
}
