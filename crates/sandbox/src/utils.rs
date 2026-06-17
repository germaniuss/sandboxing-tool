use std::os::fd::{AsFd, AsRawFd};
use std::{
    hint::unreachable_unchecked,
    os::fd::{FromRawFd as _, OwnedFd, RawFd},
};

use nix::unistd::Pid;
use nix::{NixPath, Result, errno::Errno, libc};

// PIDFD_OPEN

libc_bitflags! {
    pub struct PidfdFlags: libc::c_uint {
        PIDFD_NONBLOCK;
        PIDFD_THREAD;
    }
}

pub fn pidfd_open(pid: Pid, flags: PidfdFlags) -> Result<OwnedFd> {
    unsafe {
        match libc::syscall(libc::SYS_pidfd_open, pid.as_raw(), flags.bits()) {
            fd if fd >= 0 => Ok(OwnedFd::from_raw_fd(RawFd::try_from(fd).unwrap_unchecked())),
            -1 => Err(Errno::last()),
            _ => unreachable_unchecked(),
        }
    }
}

// CLONE

libc_bitflags! {
    pub struct CloneFlags: libc::c_int {
        CLONE_VM;
        CLONE_FS;
        CLONE_FILES;
        CLONE_SIGHAND;
        CLONE_PTRACE;
        CLONE_VFORK;
        CLONE_PARENT;
        CLONE_THREAD;
        CLONE_NEWNS;
        CLONE_SYSVSEM;
        CLONE_UNTRACED;
        CLONE_NEWCGROUP;
        CLONE_NEWUTS;
        CLONE_NEWIPC;
        CLONE_NEWUSER;
        CLONE_NEWPID;
        CLONE_NEWNET;
        CLONE_IO;
        CLONE_NEWTIME;
        SIGCHLD;
    }
}

impl From<nix::sched::CloneFlags> for CloneFlags {
    fn from(flags: nix::sched::CloneFlags) -> Self {
        Self::from_bits_retain(flags.bits())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CloneResult {
    Parent { child: Pid },
    Child,
}

unsafe fn sys_clone(flags: libc::c_ulong, child_stack: *mut libc::c_void) -> libc::c_int {
    #[cfg(any(target_arch = "s390x"))]
    unsafe {
        libc::syscall(libc::SYS_clone, child_stack, flags) as libc::c_int
    }
    #[cfg(not(any(target_arch = "s390x")))]
    unsafe {
        libc::syscall(libc::SYS_clone, flags, child_stack) as libc::c_int
    }
}

pub unsafe fn clone(flags: CloneFlags, mut child_stack: Option<&mut [u8]>) -> Result<CloneResult> {
    let stack_ptr = match child_stack {
        Some(ref mut stack) => {
            let end_ptr = unsafe { stack.as_mut_ptr().add(stack.len()) };
            end_ptr as *mut libc::c_void
        }
        None => std::ptr::null_mut(),
    };

    match unsafe { sys_clone(flags.bits() as libc::c_ulong, stack_ptr) } {
        0 => Ok(CloneResult::Child),
        -1 => Err(Errno::from_raw(unsafe {
            std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_unchecked()
        })),
        p => Ok(CloneResult::Parent {
            child: nix::unistd::Pid::from_raw(p),
        }),
    }
}

#[allow(unused)]
pub fn unshare(flags: CloneFlags) -> Result<()> {
    let res = unsafe { libc::unshare(flags.bits()) };

    Errno::result(res).map(drop)
}

#[allow(unused)]
pub fn setns<Fd: AsFd>(fd: Fd, nstype: CloneFlags) -> Result<()> {
    let res = unsafe { libc::setns(fd.as_fd().as_raw_fd(), nstype.bits()) };

    Errno::result(res).map(drop)
}

// mount

libc_bitflags! {
    pub struct AtFlags: libc::c_int {
        AT_REMOVEDIR;
        AT_SYMLINK_FOLLOW;
        AT_SYMLINK_NOFOLLOW;
        AT_NO_AUTOMOUNT;
        AT_EMPTY_PATH;
        AT_EACCESS;
        AT_RECURSIVE;
    }
}

libc_bitflags! {
    pub struct MountAttrFlags: libc::c_ulong {
        MOUNT_ATTR_RDONLY;
        MOUNT_ATTR_NOSUID;
        MOUNT_ATTR_NODEV;
        MOUNT_ATTR_NOEXEC;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MountAttr {
    pub attr_clr: libc::c_ulong,
    attr_set: libc::c_ulong,
    pub propagation: libc::c_ulong,
    pub userns_fd: libc::c_ulong,
}

impl MountAttr {
    /// Set the flags from the strongly-typed bitflags.
    #[inline]
    pub fn set_attr_set(&mut self, flags: MountAttrFlags) {
        self.attr_set = flags.bits();
    }
}

pub fn mount_setattr<P: ?Sized + NixPath, Fd: AsFd>(
    dirfd: Fd,
    path: &P,
    flags: AtFlags,
    attr: &MountAttr,
) -> Result<()> {
    let flags: AtFlags = flags.into();
    let res = path.with_nix_path(|cstr| unsafe {
        libc::syscall(
            libc::SYS_mount_setattr,
            dirfd.as_fd().as_raw_fd() as libc::c_int,
            cstr.as_ptr(),
            flags.bits(),
            attr as *const MountAttr as *const libc::c_void,
            std::mem::size_of::<libc::mount_attr>() as libc::size_t,
        )
    })?;

    Errno::result(res)?;
    Ok(())
}

libc_bitflags! {
    pub struct OpenTreeFlags: libc::c_uint {
        AT_EMPTY_PATH as libc::c_uint;
        AT_NO_AUTOMOUNT as libc::c_uint;
        AT_SYMLINK_NOFOLLOW as libc::c_uint;
        OPEN_TREE_CLOEXEC;
        OPEN_TREE_CLONE;
        AT_RECURSIVE as libc::c_uint;
    }
}

#[allow(unused)]
pub fn open_tree<P: ?Sized + NixPath, Fd: AsFd>(
    fd: Fd,
    path: &P,
    flags: OpenTreeFlags,
) -> Result<OwnedFd> {
    let fd = path.with_nix_path(|cstr| unsafe {
        libc::syscall(
            libc::SYS_open_tree,
            fd.as_fd().as_raw_fd(),
            cstr.as_ptr(),
            flags,
        )
    })?;
    Errno::result(fd)?;

    Ok(unsafe { OwnedFd::from_raw_fd(fd as libc::c_int) })
}

libc_bitflags! {
    pub struct MoveMountFlags: libc::c_uint {
        MOVE_MOUNT_F_EMPTY_PATH;
        MOVE_MOUNT_T_EMPTY_PATH;
        MOVE_MOUNT_F_SYMLINKS;
        MOVE_MOUNT_T_SYMLINKS;
        MOVE_MOUNT_F_AUTOMOUNTS;
        MOVE_MOUNT_T_AUTOMOUNTS;
        MOVE_MOUNT_SET_GROUP;
        MOVE_MOUNT_BENEATH;
    }
}

#[allow(unused)]
pub fn move_mount<P1, Fd1, P2, Fd2>(
    from_fd: Fd1,
    from_path: &P1,
    to_fd: Fd2,
    to_path: &P2,
    flags: MoveMountFlags,
) -> Result<OwnedFd>
where
    P1: ?Sized + NixPath,
    Fd1: AsFd,
    P2: ?Sized + NixPath,
    Fd2: AsFd,
{
    let fd = from_path.with_nix_path(|from_cstr| {
        to_path.with_nix_path(|to_cstr| unsafe {
            libc::syscall(
                libc::SYS_move_mount,
                from_fd.as_fd().as_raw_fd(),
                from_cstr.as_ptr(),
                to_fd.as_fd().as_raw_fd(),
                to_cstr.as_ptr(),
                flags.bits(),
            ) as libc::c_int
        })
    })??;
    Errno::result(fd)?;

    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}
