mod utils;

use std::{
    io,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use crate::{
    fs::{ensure_dir, ensure_file, ensure_parent},
    operations::utils::{BindOptions, bind, dev},
};

pub struct Bind {
    pub source: PathBuf,
    pub target: PathBuf,
    pub readonly: bool,
    pub device: bool,
    pub recursive: bool,
}

pub struct Symlink {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Default)]
pub struct Proc {
    pub source: Option<PathBuf>,
    pub target: Option<PathBuf>,
}

#[derive(Default)]
pub struct Dev {
    pub source: Option<PathBuf>,
    pub target: Option<PathBuf>,
}

pub enum Operation {
    Bind(Bind),
    Symlink(Symlink),
    Proc(Proc),
    Dev(Dev),
}

fn strip_prefix<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref()
        .strip_prefix("/")
        .unwrap_or(path.as_ref())
        .to_path_buf()
}

impl Operation {
    fn ensure<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        old: P1,
        new: P2,
    ) -> io::Result<(PathBuf, PathBuf)> {
        let old = old.as_ref();
        let new = new.as_ref();
        match self {
            Operation::Bind(Bind { source, target, .. }) => {
                let source = old.join(strip_prefix(source));
                let target = new.join(strip_prefix(target));
                ensure_parent(&target, 0o755)?;
                if source.is_dir() {
                    ensure_dir(&target, 0o755)?;
                } else {
                    ensure_file(&target, 0o444)?;
                }
                Ok((source, target))
            }
            Operation::Symlink(Symlink { source, target }) => {
                let source = source.join("");
                let target = new.join(strip_prefix(target));
                ensure_parent(&target, 0o755)?;
                Ok((source, target))
            }
            Operation::Proc(Proc { source, target }) => {
                let source = old.join(strip_prefix(source.clone().unwrap_or("proc".into())));
                let target = new.join(strip_prefix(target.clone().unwrap_or("proc".into())));
                // we should check that both src and target are procfs
                ensure_parent(&target, 0o755)?;
                ensure_dir(&target, 0o755)?;
                Ok((source, target))
            }
            Operation::Dev(Dev { source, target }) => {
                let source = old.join(strip_prefix(source.clone().unwrap_or("dev".into())));
                let target = new.join(strip_prefix(target.clone().unwrap_or("dev".into())));
                ensure_parent(&target, 0o755)?;
                ensure_dir(&target, 0o755)?;
                Ok((source, target))
            }
        }
    }
}

pub(crate) fn apply<P1: AsRef<Path>, P2: AsRef<Path>>(
    operations: &Vec<Operation>,
    old: P1,
    new: P2,
) -> io::Result<()> {
    for op in operations {
        let (source, target) = op.ensure(&old, &new)?;
        match op {
            Operation::Bind(Bind {
                readonly,
                device,
                recursive,
                ..
            }) => {
                let mut bind_flags = BindOptions::empty();
                bind_flags.set(BindOptions::BIND_READONLY, *readonly);
                bind_flags.set(BindOptions::BIND_DEVICES, *device);
                bind_flags.set(BindOptions::BIND_RECURSIVE, *recursive);
                bind(Some(source), target, bind_flags, true)?;
            }
            Operation::Symlink(Symlink { .. }) => symlink(&source, &target)?,
            Operation::Proc(Proc { .. }) => {
                bind(Some(&source), &target, BindOptions::BIND_RECURSIVE, false)?
            }
            Operation::Dev(Dev { .. }) => dev(&source, &target)?,
        }
    }
    Ok(())
}
