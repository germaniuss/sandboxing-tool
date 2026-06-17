use std::fs::DirBuilder;
use std::os::unix::fs::{DirBuilderExt, symlink};
use std::path::PathBuf;
use std::{io, path::Path};

use nix::NixPath;
use nix::fcntl::{OFlag, open};
use nix::mount::{MsFlags, mount};
use nix::sys::stat::Mode;

use crate::fs::create_file;
use crate::utils::{AtFlags, MountAttr, MountAttrFlags, mount_setattr};

bitflags::bitflags! {
    pub(super) struct BindOptions: u32 {
        const BIND_READONLY      = 1 << 0;
        const BIND_DEVICES       = 1 << 1;
        const BIND_RECURSIVE     = 1 << 2;
    }
}

pub fn bind<P1: AsRef<Path>, P2: AsRef<Path>>(
    src: Option<P1>,
    dest: P2,
    options: BindOptions,
    remount: bool,
) -> io::Result<()> {
    let readonly = options.contains(BindOptions::BIND_READONLY);
    let devices = options.contains(BindOptions::BIND_DEVICES);
    let recursive = options.contains(BindOptions::BIND_RECURSIVE);

    if let Some(source) = src {
        let mut flags = MsFlags::MS_SILENT | MsFlags::MS_BIND;
        if recursive {
            flags |= MsFlags::MS_REC;
        }
        mount(
            Some(source.as_ref()),
            dest.as_ref(),
            None::<&str>,
            flags,
            None::<&str>,
        )?;
    }

    if remount {
        let dest = dest.as_ref().canonicalize()?;
        let dest_fd = open(&dest, OFlag::O_PATH | OFlag::O_CLOEXEC, Mode::empty())?;

        let mut attr = MountAttr::default();
        let mut mount_attr_flags = MountAttrFlags::MOUNT_ATTR_NOSUID;

        if !devices {
            mount_attr_flags |= MountAttrFlags::MOUNT_ATTR_NODEV
        }
        if readonly {
            mount_attr_flags |= MountAttrFlags::MOUNT_ATTR_RDONLY
        }

        let mut at_flags = AtFlags::AT_EMPTY_PATH;

        if recursive {
            at_flags |= AtFlags::AT_RECURSIVE;
        }

        attr.set_attr_set(mount_attr_flags);
        mount_setattr(dest_fd, "", at_flags, &attr)?;
    }

    Ok(())
}

pub fn tmpfs<P: ?Sized + NixPath>(dest: &P, mode: u32, size: u32) -> io::Result<()> {
    let opt = match size {
        0 => format!("mode={:04o}", mode),
        _ => format!("mode={:04o},size={}", mode, size),
    };
    mount(
        Some("tmpfs"),
        dest,
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
        Some(opt.as_str()),
    )?;

    Ok(())
}

pub fn dev<P1: ?Sized + AsRef<Path>, P2: ?Sized + AsRef<Path>>(
    src: &P1,
    dest: &P2,
) -> io::Result<()> {
    let src = src.as_ref();
    let dest = dest.as_ref();

    tmpfs(dest, 0o755, 0)?;

    let devnodes = ["null", "zero", "full", "random", "urandom", "tty"];
    for node in devnodes {
        let src = src.join(node);
        let dest = dest.join(node);
        create_file(&dest, 0o444, None).map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Can't create file {}", dest.display()),
            )
        })?;
        bind(Some(&src), &dest, BindOptions::BIND_DEVICES, false)?;
    }

    let stdionodes = ["stdin", "stdout", "stderr"];
    for (fd, node) in stdionodes.iter().enumerate() {
        let src: PathBuf = format!("/proc/self/fd/{fd}").into();
        let dest = dest.join(node);
        symlink(&src, &dest)?;
    }

    symlink("/proc/self/fd", dest.join("fd"))?;
    symlink("/proc/kcore", dest.join("core"))?;
    DirBuilder::new().mode(0o755).create(dest.join("shm"))?;
    DirBuilder::new().mode(0o755).create(dest.join("pts"))?;

    mount(
        Some("devpts"),
        &dest.join("pts"),
        Some("devpts"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC,
        Some("newinstance,ptmxmode=0666,mode=620"),
    )?;

    symlink("pts/ptmx", dest.join("ptmx"))?;

    Ok(())
}
