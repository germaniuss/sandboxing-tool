use rustix::process::PidfdFlags;
use systemd_bus::{self as systemd};
use zbus::zvariant;

use crate::profile::Profile;

use rustix::process::{getpid, pidfd_open};
use std::fmt::Write;
use std::{env, io};

fn name(name: &str) -> String {
    let mut unit_prefix = String::with_capacity(255);
    unit_prefix.push_str("app-");

    if let Ok(xdg_curr_desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop_part = xdg_curr_desktop
            .split(':')
            .next()
            .unwrap_or(&xdg_curr_desktop);
        unit_prefix.push_str(desktop_part);
        unit_prefix.push('-');
    }

    unit_prefix.push_str(name);

    let mut prefix_bytes = unit_prefix.into_bytes();
    for byte in &mut prefix_bytes {
        let is_valid =
            byte.is_ascii_alphanumeric() || matches!(*byte, b':' | b'-' | b'_' | b'.' | b'\\');

        if !is_valid {
            *byte = b'_';
        }
    }

    prefix_bytes.truncate(220);

    let mut final_name = unsafe { String::from_utf8_unchecked(prefix_bytes) };

    let mut bytes = [0u8; 8];
    let mut rng = fastrand::Rng::new();
    rng.fill(&mut bytes);

    let rand_u64 = u64::from_ne_bytes(bytes);

    write!(&mut final_name, "-{:016x}.scope", rand_u64)
        .expect("Writing to a pre-allocated String should never fail");

    final_name
}

fn properties<'a>(profile: &Profile, pidfd: zvariant::Fd<'a>) -> systemd::properties::Scope<'a> {
    let mut memory_max = None;
    let mut memory_swap_max = None;
    let mut cpu_quota_per_sec_usec = None;

    let mut timeout_stop_usec = None;
    let mut runtime_max_usec = None;

    if let Some(p_resources) = &profile.resources {
        if p_resources.memory.is_some() {
            memory_max = p_resources.memory;
            memory_swap_max = Some(0);
        };
        cpu_quota_per_sec_usec = p_resources.cpus;
    };

    if let Some(timeout) = profile.timeout {
        timeout_stop_usec = Some(1_000_000 * timeout);
        runtime_max_usec = Some(1_000_000 * timeout);
    }

    let unit = systemd::properties::Unit::builder()
        .collect_mode("inactive-or-failed")
        .build();

    // Setup resource limits
    let resources = systemd::properties::Resources::builder()
        .slice("app.slice")
        .maybe_memory_max(memory_max)
        .maybe_memory_swap_max(memory_swap_max)
        .maybe_cpu_quota_per_sec_usec(cpu_quota_per_sec_usec)
        .build();

    // Setup scope properties
    let scope = systemd::properties::Scope::builder()
        .unit(unit)
        .resources(resources)
        .maybe_timeout_stop_usec(timeout_stop_usec)
        .maybe_runtime_max_usec(runtime_max_usec)
        .pidfds(Box::new([pidfd]))
        .build();

    scope
}

pub async fn start(profile: &Profile) -> zbus::Result<()> {
    // we use pidfd to avoid race conditions
    let fd = pidfd_open(getpid(), PidfdFlags::empty()).map_err(|e| io::Error::from(e))?;
    let properties = properties(&profile, fd.into());
    let name = name(&profile.command);
    systemd::start(&name, "replace", Some(properties), 0, None).await
}
