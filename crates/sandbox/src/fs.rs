use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::Path,
};

pub fn ensure_parent<P: AsRef<Path>>(path: P, mode: u32) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if parent.is_dir() {
            return Ok(());
        }
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(mode)
            .create(parent)?;
    }
    Ok(())
}

pub fn ensure_dir<P: AsRef<Path>>(path: P, mode: u32) -> io::Result<()> {
    if path.as_ref().is_dir() {
        return Ok(());
    }
    std::fs::DirBuilder::new().mode(mode).create(path)?;
    Ok(())
}

pub fn ensure_file<P: AsRef<Path>>(path: P, mode: u32) -> io::Result<()> {
    let path = path.as_ref();

    if let Ok(metadata) = fs::metadata(path) {
        if !metadata.is_dir() && !metadata.is_symlink() {
            return Ok(());
        }
    }

    match create_file(&path, mode, None) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }
}

fn _write(file: &mut File, content: Option<&str>) -> io::Result<()> {
    if let Some(text) = content {
        file.write_all(text.as_bytes())
    } else {
        Ok(())
    }
}

pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .truncate(true)
        .open(path)?;
    _write(&mut file, Some(content))
}

pub fn create_file<P: AsRef<Path>>(path: P, mode: u32, content: Option<&str>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(path)?;
    _write(&mut file, content)
}
