mod profile;
mod scope;

use std::{
    error::Error,
    io::{self, IsTerminal},
    os::fd::OwnedFd,
    path::PathBuf,
};

use nix::unistd::{Gid, Uid};
use sandbox::operations;

fn try_get_stdio(
    stream_opt: Option<profile::Stdio>,
    attach: bool,
) -> io::Result<sandbox::process::Stdio> {
    if let Some(stream) = stream_opt {
        Ok(stream.try_into()?)
    } else {
        if !attach {
            Ok(sandbox::process::Stdio::null())
        } else {
            Ok(sandbox::process::Stdio::inherit())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let profile = profile::profile()?;
    scope::start(&profile).await?;

    // --wait:                          attach stdin, stdout and stderr                                 ->      operates as a normal command
    // none:                            attach stdin if not a terminal, detach stdout and stderr        ->      detaches and returns the pid of the running process to the terminal
    // --stdin/stdout/stderr=inherit    attach stdio stream, overwrites the defaults
    // --stdin/stdout/stderr=/file      attach stdio stream to a file, overwrites the defaults

    let mut stdin_fifo: Option<OwnedFd> = None;
    let mut stdin_path: Option<PathBuf> = None;
    let mut stdin_remove: bool = false;

    // We keep a fifo open with RDWR so that stdin does not
    // close when used from a fifo file
    let stdin = if let Some(stdin) = profile.stdio.stdin {
        Some(match stdin {
            crate::profile::Stdio::File {
                fifo,
                path,
                created,
                file,
            } => {
                stdin_fifo = fifo;
                if created && profile.wait {
                    stdin_remove = true;
                }
                if created {
                    stdin_path = Some(path);
                }
                crate::profile::Stdio::File {
                    file,
                    path: "".into(),
                    created,
                    fifo: None,
                }
            }
            _ => stdin,
        })
    } else {
        profile.stdio.stdin
    };

    let stdin = try_get_stdio(stdin, profile.wait || !io::stdin().is_terminal())?;
    let stdout = try_get_stdio(profile.stdio.stdout, profile.wait)?;
    let stderr = try_get_stdio(profile.stdio.stderr, profile.wait)?;

    let die_with_parent_thread = if profile.wait {
        // kill current parent process also when its parent dies
        sandbox::die_with_parent_thread(None::<&OwnedFd>)?;
        true
    } else {
        false
    };

    let binds: Vec<operations::Bind> = profile.operations.binds.iter().map(Into::into).collect();
    let devices: Vec<operations::Bind> =
        profile.operations.devices.iter().map(Into::into).collect();
    let symlinks: Vec<operations::Symlink> =
        profile.operations.symlinks.iter().map(Into::into).collect();

    // I should try to add a new clone flags to the kernel
    // reusing CLONE_DETACHED it would function as follows:
    //
    // When used it will spawn the new process with its parent
    // being the closest subreaper ancestor. When used after setns
    // with CLONE_NEWPID the newly spawned process will attach itself
    // to PID 1 if it exists or become PID 1 if a PID 1 process
    // does not exist. This functionality is useful in the following
    // situations. The PID of the process in the original namespace
    // is returned
    //
    // 1. We need to spawn a process that may finish before its original
    // parent and need to be awaited while the parent continues execution,
    // that it we need a detached process, normally to create a scuh a process
    // we would need  to do a double fork or create a new thread to await the
    // spawned process. Using this flag we can avoid the need to create extra
    // processes/threads to spawn a detached process.
    // 2. When spawning a process in an existing PID namespace it can be useful to spawn
    // a process directly attached to PID 1 as its parent. Currently when we use
    // setns to attach the child to a PID namespace its parent in the PID namespace
    // is PID 0, this can cause the PID namespace cleanup to hang when PID 1 dies
    // since the spawned process needs to be waited on outside the PID namespace.
    // In order to avoid this the only way to currently achieve it is to do a double
    // fork inside the PID namespace so that the grandchild reparents to PID 1. This
    // creates unnecessary short lived processes in a PID namespace and can fail when
    // process limits exists in a sandbox. Another painpoint is the difficulty to return
    // the real PID (non PID namespace one) to the spawning process after a double fork
    // (even more difficult for a pidfd when used with CLONE_PIDFD)
    // 3. Currently it is impossible to spawn a PID namespace with a helper child process
    // and hold it with an open fd on /proc/[pid]/ns/pid even after the child exists. The
    // reason is that the helper child becomes PID 1 and any further process we attach to
    // the empty namespace will have PID 2 onwards so an init process will not exist. This
    // makes it much more difficult to setup a sandbox/container before PID 1 is spawned.
    // Setting up the container with a child process before spawning the final PID 1 can
    // be useful to do setup with elevated privileges. It also allows to simplify the code
    // needed for spawning processes in a PID namespace as PID 1 and subsequent attached
    // processes do not have to be treated differently. Currently PID 1 has to be spawned
    // with a single clone and further processes require a double fork, so some variable is
    // required to track whether an init process has been spawned, another option is to check
    // the pid numbers in /proc. Both of these alternatives are difficult to handle cleanly
    // and can be racy is handled improperly. This becomes even more difficult if we are in
    // a multi process program where the first configures the sandbox and the second wants to
    // spawn the init process as then some multi process syncronization is required.

    // SandboxBuilder::build will create and
    // configure all the namespaces with the
    // exception of the pid namespace, the pid
    // namespace will be configured by the PID1
    // process. The created namespaces persist
    // as long as Sandbox struct is not dropped
    let container = sandbox::Sandbox::builder() // allow specifying capabilities
        .unshare_user(true)
        .unshare_mount(true)
        .unshare_pid(true)
        .unshare_cgroup(true)
        .unshare_ipc(true)
        .unshare_uts(true)
        .newroot(true)
        .max_user_namespaces(0)
        .max_mnt_namespaces(0)
        .max_pid_namespaces(1)
        .max_cgroup_namespaces(0)
        .max_ipc_namespaces(0)
        .max_uts_namespaces(0)
        .max_net_namespaces(0)
        .max_time_namespaces(0)
        .uid(Uid::current())
        .gid(Gid::current())
        .binds(binds)
        .binds(devices)
        .symlinks(symlinks)
        .proc(operations::Proc::default())
        .dev(operations::Dev::default());

    let mut container = container.build()?;

    // spawn child in a new
    // pid namespace as pid1
    let child = container
        .new(&profile.command)
        .args(&profile.args)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(stderr)
        .die_with_parent_thread(die_with_parent_thread)
        .kill_on_drop(profile.wait)
        .reap_on_drop(profile.wait)
        .spawn()?;

    if profile.wait {
        child.wait()?;
    } else {
        println!("{}", child.getpid());
    }

    drop(stdin_fifo);
    if stdin_remove {
        let _: io::Result<()> = stdin_path.map_or(Ok(()), |p| {
            std::fs::remove_file(p)?;
            Ok(())
        });
    }

    Ok(())
}
