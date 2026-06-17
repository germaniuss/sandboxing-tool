#[macro_use]
mod macros;

mod fs;
pub mod operations;
pub mod process;
pub mod sandbox;
mod utils;

pub use sandbox::*;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use super::*;
    use rstest::*;

    use nix::{
        mount::{MsFlags, mount},
        unistd::{Gid, Pid, Uid, getpid},
    };

    fn inode(name: &str) -> std::io::Result<String> {
        let path = format!("/proc/self/ns/{name}");
        std::fs::read_link(path).map(|path| path.to_string_lossy().into_owned())
    }

    fn ms(ms: u32) -> Duration {
        Duration::from_millis(ms.into())
    }

    #[fixture]
    fn tmpdir() -> std::path::PathBuf {
        let mut bytes = [0u8; 8];
        fastrand::fill(&mut bytes);
        let rand_u64 = u64::from_ne_bytes(bytes);
        let path = std::env::temp_dir().join(format!("sandbox_test_{:016x}", rand_u64));
        let _ = std::fs::create_dir_all(&path);
        path
    }

    type SandboxBuilderType = SandboxBuilder<
        sandbox_builder::SetNewroot<sandbox_builder::SetGid<sandbox_builder::SetUid>>,
    >;

    #[fixture]
    fn sandbox(#[default(&["user"])] unshare_ns: &[&str]) -> SandboxBuilderType {
        let mut builder = Sandbox::builder()
            .uid(Uid::from_raw(0))
            .gid(Gid::from_raw(0))
            .newroot(true);
        for ns in unshare_ns.to_vec() {
            builder = match ns {
                "user" => builder.unshare_user(true),
                "mnt" => builder.unshare_mount(true),
                "pid" => builder.unshare_pid(true),
                "cgroup" => builder.unshare_cgroup(true),
                "ipc" => builder.unshare_ipc(true),
                "uts" => builder.unshare_uts(true),
                _ => builder,
            };
        }
        builder
    }

    #[rstest]
    #[case("user")]
    #[case("mnt")]
    #[case("pid")]
    #[case("cgroup")]
    #[case("ipc")]
    #[case("uts")]
    // #[timeout(ms(1000))]
    fn test_ns_unshare(
        #[case] ns: &str,
        #[with(&["user", ns])] sandbox: SandboxBuilderType,
    ) -> std::io::Result<()> {
        let mut sandbox = sandbox.proc(operations::Proc::default()).build()?;

        let inode_ = inode(ns)?;
        let status = sandbox
            .run(|| {
                assert_ne!(inode_, inode(ns)?);
                Ok(())
            })?
            .wait()?;

        assert!(status.success());
        Ok(())
    }

    // test user namespace

    #[rstest]
    #[case(0, 0)]
    #[case(Uid::current().as_raw(), Gid::current().as_raw())]
    // #[timeout(ms(1000))]
    fn test_ns_user_map(
        #[case] uid: u32,
        #[case] gid: u32,
        #[with(&["user"])] sandbox: SandboxBuilderType,
    ) -> std::io::Result<()> {
        let mut sandbox = sandbox.uid(Uid::from(uid)).gid(Gid::from(gid)).build()?;

        let status = sandbox
            .run(|| {
                assert_eq!(Uid::current().as_raw(), uid);
                assert_eq!(Gid::current().as_raw(), gid);
                Ok(())
            })?
            .wait()?;

        assert!(status.success());
        Ok(())
    }

    // test mount namespace

    #[rstest]
    // #[timeout(ms(1000))]
    fn test_ns_mnt_empty(
        #[with(&["user", "mnt"])] sandbox: SandboxBuilderType,
    ) -> std::io::Result<()> {
        let mut sandbox = sandbox.newroot(true).build()?;

        let status = sandbox
            .run(|| {
                let dir = std::env::current_dir()?;
                let empty = dir.read_dir()?.next().is_none();
                assert!(empty, "Mount namespace is not empty");
                Ok(())
            })?
            .wait()?;

        assert!(status.success());
        Ok(())
    }

    #[rstest]
    // #[timeout(ms(1000))]
    fn test_ns_mnt_isolation(
        tmpdir: std::path::PathBuf,
        // don't create a new empty root since we need /tmp to exist
        #[with(&["user", "mnt"])] sandbox: SandboxBuilderType,
    ) -> std::io::Result<()> {
        let mut sandbox = sandbox.newroot(false).build()?;

        let status = sandbox
            .run({
                let target = tmpdir.clone();
                move || {
                    mount(
                        Some("tmpfs"),
                        &target,
                        Some("tmpfs"),
                        MsFlags::empty(),
                        None::<&str>,
                    )?;
                    std::fs::write(target.join("isolated.txt"), b"data")?;
                    Ok(())
                }
            })?
            .wait()?;

        assert!(
            !tmpdir.join("isolated.txt").exists(),
            "Mount leaked to the host filesystem"
        );

        assert!(status.success());
        Ok(())
    }

    #[rstest]
    // #[timeout(ms(1000))]
    fn test_ms_mnt_bind(
        #[with(&["user", "mnt"])] sandbox: SandboxBuilderType,
    ) -> std::io::Result<()> {
        let mut sandbox = sandbox
            .bind(operations::Bind {
                source: "/usr".into(),
                target: "/usr".into(),
                readonly: true,
                device: false,
                recursive: true,
            })
            .build()?;

        let status = sandbox
            .run(|| {
                let target: std::path::PathBuf = "/usr".into();
                assert!(target.exists(), "Directory was not bind mounted in sandbox");
                Ok(())
            })?
            .wait()?;

        assert!(status.success());
        Ok(())
    }

    // test pid namespace

    #[rstest]
    // #[timeout(ms(1000))]
    fn test_ns_pid_pid1(
        #[with(&["user", "pid"])] sandbox: SandboxBuilderType,
    ) -> std::io::Result<()> {
        let mut sandbox = sandbox.build()?;

        let status = sandbox
            .run(|| {
                assert_eq!(Pid::this().as_raw(), 1, "Process ID should be 1");
                Ok(())
            })?
            .wait()?;

        assert!(status.success());
        Ok(())
    }

    // test cgroup namespace

    #[rstest]
    // #[timeout(ms(1000))]
    fn test_ns_cgroup_root_empty(
        #[with(&["user", "cgroup"])] sandbox: SandboxBuilderType,
    ) -> std::io::Result<()> {
        let mut sandbox = sandbox.build()?;

        let status = sandbox
            .run(|| {
                let cgroup_data = std::fs::read_to_string("/proc/self/cgroup")?;
                assert!(cgroup_data.contains(":/"), "Cgroup root was not unshared");
                Ok(())
            })?
            .wait()?;

        assert!(status.success());
        Ok(())
    }

    // test ipc namespace

    // test uts namespace
}
