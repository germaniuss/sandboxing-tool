use crate::fs::write_file;
use crate::operations;
use crate::process::{Child, Command, null_file};
use crate::utils::{CloneFlags, CloneResult, PidfdFlags, clone, pidfd_open, setns};

use std::ffi::OsStr;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use bon::{Builder, builder};
use nix::errno::Errno;
use nix::fcntl::{OFlag, open};
use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use nix::sys::eventfd::EventFd;
use nix::sys::stat::{Mode, umask};
pub use nix::sys::wait::waitpid;
use nix::sys::wait::{WaitPidFlag, WaitStatus};
use nix::unistd::{Gid, Pid, Uid, chdir, close, fchdir, getpid, pipe2, pivot_root};

#[derive(Builder)]
#[builder(finish_fn(vis = "", name = _build))]
#[builder(state_mod(vis = "pub(crate)"))]
#[builder(on(_, overwritable))]
pub struct Sandbox {
    #[builder(field = CloneFlags::SIGCHLD)]
    flags: CloneFlags,
    #[builder(field)]
    operations: Vec<operations::Operation>,
    #[builder(skip)]
    fds_ns: Vec<(CloneFlags, OwnedFd)>,

    uid: Uid,
    gid: Gid,

    newroot: bool,

    max_user_namespaces: Option<i32>,
    max_mnt_namespaces: Option<i32>,
    max_pid_namespaces: Option<i32>,
    max_cgroup_namespaces: Option<i32>,
    max_net_namespaces: Option<i32>,
    max_ipc_namespaces: Option<i32>,
    max_uts_namespaces: Option<i32>,
    max_time_namespaces: Option<i32>,
}

impl<S: sandbox_builder::State> SandboxBuilder<S> {
    pub fn unshare_user(mut self, value: bool) -> Self {
        self.flags.set(CloneFlags::CLONE_NEWUSER, value);
        self
    }
    pub fn unshare_mount(mut self, value: bool) -> Self {
        self.flags.set(CloneFlags::CLONE_NEWNS, value);
        self
    }
    pub fn unshare_pid(mut self, value: bool) -> Self {
        self.flags.set(CloneFlags::CLONE_NEWPID, value);
        self
    }
    pub fn unshare_cgroup(mut self, value: bool) -> Self {
        self.flags.set(CloneFlags::CLONE_NEWCGROUP, value);
        self
    }
    pub fn unshare_ipc(mut self, value: bool) -> Self {
        self.flags.set(CloneFlags::CLONE_NEWIPC, value);
        self
    }
    pub fn unshare_uts(mut self, value: bool) -> Self {
        self.flags.set(CloneFlags::CLONE_NEWUTS, value);
        self
    }
    // pub fn unshare_net(mut self, value: bool) -> Self {
    //     self.flags.set(CloneFlags::CLONE_NEWNET, value);
    //     self
    // }
    // pub fn unshare_time(mut self, value: bool) -> Self {
    //     self.flags.set(CloneFlags::CLONE_NEWTIME, value);
    //     self
    // }
}

impl<S: sandbox_builder::State> SandboxBuilder<S> {
    pub fn bind(mut self, bind: operations::Bind) -> Self {
        self.operations.push(operations::Operation::Bind(bind));
        self
    }
    pub fn binds(mut self, binds: Vec<operations::Bind>) -> Self {
        self.operations
            .extend(binds.into_iter().map(|b| operations::Operation::Bind(b)));
        self
    }
}

impl<S: sandbox_builder::State> SandboxBuilder<S> {
    pub fn symlink(mut self, symlink: operations::Symlink) -> Self {
        self.operations
            .push(operations::Operation::Symlink(symlink));
        self
    }
    pub fn symlinks(mut self, symlinks: Vec<operations::Symlink>) -> Self {
        self.operations.extend(
            symlinks
                .into_iter()
                .map(|s| operations::Operation::Symlink(s)),
        );
        self
    }
}

impl<S: sandbox_builder::State> SandboxBuilder<S> {
    pub fn proc(mut self, proc: operations::Proc) -> Self {
        self.operations.push(operations::Operation::Proc(proc));
        self
    }
}

impl<S: sandbox_builder::State> SandboxBuilder<S> {
    pub fn dev(mut self, dev: operations::Dev) -> Self {
        self.operations.push(operations::Operation::Dev(dev));
        self
    }
}

impl<S: sandbox_builder::IsComplete> SandboxBuilder<S> {
    pub fn build(self) -> io::Result<Sandbox> {
        let mut container = self._build();
        container._build()?;
        Ok(container)
    }
}

impl Sandbox {
    fn setup_ns_mount(&self, _: &Context) -> io::Result<()> {
        if self.flags.contains(CloneFlags::CLONE_NEWNS) {
            let base: PathBuf = "/tmp".into();
            let old_root: PathBuf = "oldroot".into();
            let new_root: PathBuf = "newroot".into();

            let old_umask = umask(Mode::empty());

            mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_SILENT | MsFlags::MS_SLAVE | MsFlags::MS_REC,
                None::<&str>,
            )?;

            if !self.newroot {
                chdir("/")?;
                unsafe { env::set_var("PWD", "/") };
                return Ok(());
            }

            mount(
                Some("tmpfs"),
                &base,
                Some("tmpfs"),
                MsFlags::MS_NODEV | MsFlags::MS_NOSUID,
                None::<&str>,
            )?;

            std::fs::DirBuilder::new()
                .mode(0o755)
                .create(&base.join(&new_root))?;

            mount(
                Some(&base.join(&new_root)),
                &base.join(&new_root),
                None::<&str>,
                MsFlags::MS_SILENT | MsFlags::MS_MGC_VAL | MsFlags::MS_BIND | MsFlags::MS_REC,
                None::<&str>,
            )?;

            std::fs::DirBuilder::new()
                .mode(0o755)
                .create(&base.join(&old_root))?;

            let oldroot_fd = open(&base, OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty())?;

            change_root(&base, &base.join(&old_root))?;

            operations::apply(&self.operations, &old_root, &new_root)?;

            mount(
                Some(&old_root),
                &old_root,
                None::<&str>,
                MsFlags::MS_SILENT | MsFlags::MS_REC | MsFlags::MS_PRIVATE,
                None::<&str>,
            )?;

            umount2(&old_root, MntFlags::MNT_DETACH)?;

            change_root(&new_root, &new_root)?;

            fchdir(oldroot_fd.as_fd())?;

            umount2(".", MntFlags::MNT_DETACH)?;

            close(oldroot_fd)?;

            umask(old_umask);

            chdir("/")?;
            unsafe { env::set_var("PWD", "/") };
        }
        Ok(())
    }

    fn setup_ns_user(&self, ctx: &Context) -> io::Result<()> {
        if self.flags.contains(CloneFlags::CLONE_NEWUSER) {
            let parent_uid = ctx.uid.as_raw();
            let parent_gid = ctx.gid.as_raw();
            let sandbox_uid = self.uid;
            let sandbox_gid = self.gid;
            write_uid_map(sandbox_uid.as_raw(), parent_uid, false, ctx.overflow_uid)?;
            write_gid_map(
                sandbox_gid.as_raw(),
                parent_gid,
                true,
                false,
                ctx.overflow_gid,
            )?;
            write_max_ns("max_user_namespaces", self.max_user_namespaces)?;
            write_max_ns("max_mnt_namespaces", self.max_mnt_namespaces)?;
            write_max_ns("max_pid_namespaces", self.max_pid_namespaces)?;
            write_max_ns("max_cgroup_namespaces", self.max_cgroup_namespaces)?;
            write_max_ns("max_net_namespaces", self.max_net_namespaces)?;
            write_max_ns("max_ipc_namespaces", self.max_ipc_namespaces)?;
            write_max_ns("max_uts_namespaces", self.max_uts_namespaces)?;
            write_max_ns("max_time_namespaces", self.max_time_namespaces)?;
        }
        Ok(())
    }

    fn setup_ns_pid(&self, _: &Context) -> io::Result<()> {
        if self.flags.contains(CloneFlags::CLONE_NEWPID)
            && self.flags.contains(CloneFlags::CLONE_NEWNS)
            && getpid() == Pid::from_raw(1)
        {
            if let Some(proc) = self.operations.iter().find_map(|x| match x {
                operations::Operation::Proc(p) => Some(p),
                _ => None,
            }) {
                let dest: PathBuf = proc.target.clone().unwrap_or("/proc".into());
                mount(
                    Some("proc"),
                    &dest,
                    Some("proc"),
                    MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
                    None::<&str>,
                )?;
            }
        }
        Ok(())
    }
}

impl Sandbox {
    pub(crate) fn pid1_started(&self) -> bool {
        self.fds_ns
            .iter()
            .find(|&&(x, _)| x.contains(CloneFlags::CLONE_NEWPID))
            .is_some()
    }

    fn _build(&mut self) -> io::Result<()> {
        let ctx = Context::new()?;

        let mut flags = self.flags.clone();
        flags.remove(CloneFlags::CLONE_NEWPID);

        let parent_thread_fd = pidfd_open(
            getpid(),
            PidfdFlags::PIDFD_NONBLOCK | PidfdFlags::PIDFD_THREAD,
        )?;

        let event_fd = EventFd::new()?;

        match unsafe { clone(flags, None) } {
            Ok(CloneResult::Parent { child, .. }) => {
                let mut closure = || -> io::Result<()> {
                    // The order in CLONE_FLAGS is importante
                    // since when using setns we want to enter
                    // the user and mount namespaces first
                    for flag in CLONE_FLAGS {
                        if self.flags.contains(flag) && !flag.contains(CloneFlags::CLONE_NEWPID) {
                            let fd = get_ns_fd(child, flag)?;
                            self.fds_ns.push((flag, fd));
                        }
                    }

                    event_fd.write(0x06)?;
                    Ok(())
                };

                if let Err(_) = closure() {
                    self.fds_ns.clear();
                }

                drop(event_fd);
                waitpid_exit0(child, None)?;
            }
            Ok(CloneResult::Child) => {
                let closure = || -> io::Result<()> {
                    die_with_parent_thread(Some(&parent_thread_fd))?;

                    self.setup_ns_user(&ctx)?;
                    self.setup_ns_mount(&ctx)?;

                    if event_fd.read()? != 0x06 {
                        Err(io::Error::new(io::ErrorKind::InvalidData, "Bad sync byte"))
                    } else {
                        Ok(())
                    }
                };

                if let Err(e) = closure() {
                    eprintln!("Child process error: {}", e);
                    std::process::exit(1);
                }

                std::process::exit(0);
            }
            Err(e) => Err(e)?,
        }

        Ok(())
    }

    fn _execute<F>(
        &mut self,
        closure: F,
        die_with_parent_thread_: bool,
        reparent_to_pid1_in_ns: bool,
    ) -> io::Result<Pid>
    where
        F: FnOnce() -> io::Result<()>,
    {
        let ctx = Context::new()?;
        let mut die_with_parent_thread_ = die_with_parent_thread_;

        let unshare_pid = self.flags.contains(CloneFlags::CLONE_NEWPID);

        let parent_thread_fd = pidfd_open(
            getpid(),
            PidfdFlags::PIDFD_NONBLOCK | PidfdFlags::PIDFD_THREAD,
        )?;

        // we have to init the null file /dev/null
        // before we setup the container since
        // we don't know if a /dev mount will exist
        // and we need /dev/null if we are to detach
        // any standard stream for the child
        let _ = null_file()?;

        let mut flags = CloneFlags::SIGCHLD;

        if unshare_pid {
            let is_pid_1 = !self.pid1_started();

            if !is_pid_1 && reparent_to_pid1_in_ns {
                // we cannot die with the parent thread
                // as the parent thread is a throwaway
                // thread spawned in the pid namespace
                // in order to reparent the process
                // executing the command
                die_with_parent_thread_ = false;
            }

            let (read_fd, write_fd) = pipe2(OFlag::O_CLOEXEC)?;
            let mut reader = File::from(read_fd);
            let mut writer = File::from(write_fd);

            if let CloneResult::Parent { child, .. } = unsafe { clone(flags, None)? } {
                drop(writer);

                let mut buf = [0u8; 4];
                if let Err(_) = reader.read_exact(&mut buf) {
                    drop(reader);
                    let _ = waitpid(child, None);
                    return Err(io::Error::from(io::ErrorKind::BrokenPipe));
                };

                drop(reader);
                let subchild = Pid::from_raw(i32::from_ne_bytes(buf));

                if is_pid_1 {
                    let fd = get_ns_fd(subchild, CloneFlags::CLONE_NEWPID)?;
                    self.fds_ns.push((CloneFlags::CLONE_NEWPID, fd));
                }
                // we already have the subchild
                // so we don't care about any errors
                // in our direct child, so just reap
                let _ = waitpid(child, None);
                return Ok(subchild);
            }

            drop(reader);

            // set all namespaces except PID
            // user and mount are set first

            for (_, fd) in self.fds_ns.iter() {
                setns(fd, CloneFlags::empty()).unwrap_or_else(|_| std::process::exit(1));
            }

            if is_pid_1 {
                flags |= CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_PARENT;
            }

            if !is_pid_1 && !reparent_to_pid1_in_ns {
                flags |= CloneFlags::CLONE_PARENT;
            }

            if !is_pid_1
                && reparent_to_pid1_in_ns
                && let CloneResult::Parent { child, .. } =
                    unsafe { clone(flags, None).unwrap_or_else(|_| std::process::exit(1)) }
            {
                drop(writer);
                let _ = waitpid(child, None);
                std::process::exit(0);
            }

            if let CloneResult::Parent { child, .. } =
                unsafe { clone(flags, None).unwrap_or_else(|_| std::process::exit(1)) }
            {
                let bytes = child.as_raw().to_ne_bytes();
                writer
                    .write_all(&bytes)
                    .unwrap_or_else(|_| std::process::exit(1));
                drop(writer);
                std::process::exit(0);
            }
        } else if let CloneResult::Parent { child, .. } =
            unsafe { clone(flags, None).unwrap_or_else(|_| std::process::exit(1)) }
        {
            return Ok(child);
        } else {
            for (_, fd) in self.fds_ns.iter() {
                setns(fd, CloneFlags::empty()).unwrap_or_else(|_| std::process::exit(1));
            }
        }

        self.setup_ns_pid(&ctx)
            .unwrap_or_else(|_| std::process::exit(1));
        if die_with_parent_thread_ {
            die_with_parent_thread(Some(&parent_thread_fd))
                .unwrap_or_else(|_| std::process::exit(1));
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure()));
        match result {
            Ok(Ok(())) => std::process::exit(0),
            Ok(Err(_)) => std::process::exit(1),
            Err(_) => std::process::exit(101),
        }
    }

    pub fn spawn(command: &mut Command) -> io::Result<Pid> {
        command.inner._execute(
            || {
                let mut comm = std::process::Command::new(&command.program);
                comm.args(&command.args);
                comm.stdin(std::process::Stdio::try_from(&command.stdin)?);
                comm.stdout(std::process::Stdio::try_from(&command.stdout)?);
                comm.stderr(std::process::Stdio::try_from(&command.stderr)?);

                // If exec returns, it failed
                let _ = comm.exec();
                std::process::exit(1);
            },
            command.die_with_parent_thread,
            command.reparent_to_pid1_in_ns,
        )
    }

    pub fn run<F>(&mut self, closure: F) -> io::Result<Child>
    where
        F: FnOnce() -> io::Result<()>,
    {
        let pid = self._execute(closure, true, false)?;
        Ok(Child {
            pid: pid,
            kill_on_drop: true,
            reap_on_drop: true,
        })
    }

    pub fn new<'a, S: AsRef<OsStr>>(&'a mut self, program: S) -> Command<'a> {
        Command::new(self, program)
    }
}

// The ugly

pub fn waitpid_exit0(pid: Pid, flags: Option<WaitPidFlag>) -> nix::Result<WaitStatus> {
    match waitpid(pid, flags)? {
        WaitStatus::Exited(_, 0) => Ok(WaitStatus::Exited(pid, 0)),
        WaitStatus::Exited(_, _) => Err(Errno::last()),
        WaitStatus::Signaled(_, _, _) => Err(Errno::UnknownErrno),
        _ => Err(Errno::UnknownErrno),
    }
}

pub fn die_with_parent_thread<Fd: std::os::fd::AsFd>(parent_pidfd: Option<Fd>) -> io::Result<()> {
    nix::sys::prctl::set_pdeathsig(nix::sys::signal::Signal::SIGKILL)?;

    if let Some(fd) = parent_pidfd {
        let pfd = PollFd::new(fd.as_fd(), PollFlags::POLLIN);
        match poll(&mut [pfd], PollTimeout::ZERO) {
            Ok(ready) if ready > 0 => {
                std::process::exit(1);
            }
            _ => drop(fd),
        }
    }

    Ok(())
}

fn parse_sysctl(path: &str) -> io::Result<u32> {
    let buf = fs::read_to_string(path)?;
    buf.trim().parse::<u32>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid value in {path}"),
        )
    })
}

fn write_max_ns(name: &str, max_ns: Option<i32>) -> io::Result<()> {
    if let Some(max_ns) = max_ns {
        write_file(
            format!("/proc/sys/user/{}", name),
            &max_ns.to_string().as_str(),
        )?;
    }
    Ok(())
}

fn write_uid_map(
    sandbox_uid: u32,
    parent_uid: u32,
    map_root: bool,
    overflow_uid: u32,
) -> io::Result<()> {
    let uid_map = if map_root && parent_uid != 0 && sandbox_uid != 0 {
        format!("0 {overflow_uid} 1\n{sandbox_uid} {parent_uid} 1\n")
    } else {
        format!("{sandbox_uid} {parent_uid} 1\n")
    };
    write_file("/proc/self/uid_map", &uid_map)?;
    Ok(())
}

fn write_gid_map(
    sandbox_gid: u32,
    parent_gid: u32,
    deny_groups: bool,
    map_root: bool,
    overflow_gid: u32,
) -> io::Result<()> {
    let gid_map = if map_root && parent_gid != 0 && sandbox_gid != 0 {
        format!("0 {overflow_gid} 1\n{sandbox_gid} {parent_gid} 1\n")
    } else {
        format!("{sandbox_gid} {parent_gid} 1\n")
    };
    if deny_groups {
        let _ = write_file("/proc/self/setgroups", "deny\n");
    }
    write_file("/proc/self/gid_map", &gid_map)?;
    Ok(())
}

fn change_root<P>(new: P, old: P) -> nix::Result<()>
where
    P: AsRef<Path>,
{
    pivot_root(new.as_ref(), old.as_ref())?;
    chdir("/")?;
    Ok(())
}

fn get_ns_fd(pid: Pid, flag: CloneFlags) -> io::Result<OwnedFd> {
    let fd = open(
        &PathBuf::from(format!("/proc/{}/ns/{}", pid, flag.as_ns_str()?)),
        OFlag::O_CLOEXEC | OFlag::O_RDONLY,
        Mode::empty(),
    )?;
    Ok(fd)
}

pub struct Context {
    uid: Uid,
    gid: Gid,
    overflow_uid: u32,
    overflow_gid: u32,
}

impl Context {
    pub fn new() -> io::Result<Context> {
        nix::sys::prctl::set_no_new_privs()?;
        Ok(Context {
            uid: Uid::current(),
            gid: Gid::current(),
            overflow_uid: parse_sysctl("/proc/sys/kernel/overflowuid")?,
            overflow_gid: parse_sysctl("/proc/sys/kernel/overflowgid")?,
        })
    }
}

impl CloneFlags {
    fn as_ns_str(&self) -> io::Result<&'static str> {
        if self.eq(&Self::CLONE_NEWUSER) {
            Ok("user")
        } else if self.eq(&Self::CLONE_NEWCGROUP) {
            Ok("cgroup")
        } else if self.eq(&Self::CLONE_NEWIPC) {
            Ok("ipc")
        } else if self.eq(&Self::CLONE_NEWUTS) {
            Ok("uts")
        } else if self.eq(&Self::CLONE_NEWNET) {
            Ok("net")
        } else if self.eq(&Self::CLONE_NEWPID) {
            Ok("pid")
        } else if self.eq(&Self::CLONE_NEWNS) {
            Ok("mnt")
        } else if self.eq(&Self::CLONE_NEWTIME) {
            Ok("time")
        } else {
            Err(io::Error::from(ErrorKind::Unsupported))
        }
    }
}

const CLONE_FLAGS: [CloneFlags; 8] = [
    CloneFlags::CLONE_NEWUSER,
    CloneFlags::CLONE_NEWNS,
    CloneFlags::CLONE_NEWPID,
    CloneFlags::CLONE_NEWCGROUP,
    CloneFlags::CLONE_NEWNET,
    CloneFlags::CLONE_NEWIPC,
    CloneFlags::CLONE_NEWUTS,
    CloneFlags::CLONE_NEWTIME,
];
