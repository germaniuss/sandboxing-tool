use bon::Builder;
use zbus::zvariant::{Fd, Value};

macro_rules! push_opt {
    ($vec:expr, $name:expr, $opt:expr) => {
        if let Some(val) = $opt {
            $vec.push(($name, Value::from(val)));
        }
    };
}

macro_rules! extend_opt {
    ($vec:expr,$opt:expr) => {
        if let Some(val) = $opt {
            $vec.extend(Vec::<(&str, Value)>::from(val));
        }
    };
}

#[derive(Builder)]
pub struct Unit<'a> {
    description: Option<&'a str>,
    documentation: Option<&'a [&'a str]>,
    wants: Option<&'a [&'a str]>,
    requires: Option<&'a [&'a str]>,
    requisite: Option<&'a [&'a str]>,
    binds_to: Option<&'a [&'a str]>,
    part_of: Option<&'a [&'a str]>,
    upholds: Option<&'a [&'a str]>,
    conflicts: Option<&'a [&'a str]>,
    before: Option<&'a [&'a str]>,
    after: Option<&'a [&'a str]>,
    on_failure: Option<&'a [&'a str]>,
    on_success: Option<&'a [&'a str]>,
    propagates_reload_to: Option<&'a [&'a str]>,
    reload_propagated_from: Option<&'a [&'a str]>,
    propagates_stop_to: Option<&'a [&'a str]>,
    stop_propagated_from: Option<&'a [&'a str]>,
    joins_namespace_of: Option<&'a [&'a str]>,
    requires_mounts_for: Option<&'a [&'a str]>,
    wants_mounts_for: Option<&'a [&'a str]>,
    on_success_job_mode: Option<&'a str>,
    on_failure_job_mode: Option<&'a str>,
    ignore_on_isolate: Option<bool>,
    stop_when_unneded: Option<bool>,
    refuse_manual_start: Option<bool>,
    refuse_manual_stop: Option<bool>,
    allow_isolate: Option<bool>,
    default_dependencies: Option<bool>,
    survive_final_kill_signal: Option<bool>,
    collect_mode: Option<&'a str>,
    failure_action: Option<&'a str>,
    success_action: Option<&'a str>,
    failure_action_exit_status: Option<i32>,
    success_action_exit_status: Option<i32>,
    job_timeout_usec: Option<u64>,
    job_running_timeout_usec: Option<u64>,
    job_timeout_action: Option<&'a str>,
    job_timeout_reboot_argument: Option<&'a str>,
    start_limit_interval_usec: Option<u64>,
    start_limit_burst: Option<u32>,
    start_limit_action: Option<&'a str>,
    reboot_argument: Option<&'a str>,
    source_path: Option<&'a str>,
}

impl<'a> From<Unit<'a>> for Vec<(&'a str, Value<'a>)> {
    fn from(unit: Unit<'a>) -> Self {
        let mut props = Vec::new();
        push_opt!(props, "Description", unit.description);
        push_opt!(props, "Documentation", unit.documentation);
        push_opt!(props, "Wants", unit.wants);
        push_opt!(props, "Requires", unit.requires);
        push_opt!(props, "Requisite", unit.requisite);
        push_opt!(props, "BindsTo", unit.binds_to);
        push_opt!(props, "PartOf", unit.part_of);
        push_opt!(props, "Upholds", unit.upholds);
        push_opt!(props, "Conflicts", unit.conflicts);
        push_opt!(props, "Before", unit.before);
        push_opt!(props, "After", unit.after);
        push_opt!(props, "OnFailure", unit.on_failure);
        push_opt!(props, "OnSuccess", unit.on_success);
        push_opt!(props, "PropagatesReloadTo", unit.propagates_reload_to);
        push_opt!(props, "ReloadPropagatedFrom", unit.reload_propagated_from);
        push_opt!(props, "PropagatesStopTo", unit.propagates_stop_to);
        push_opt!(props, "StopPropagatedFrom", unit.stop_propagated_from);
        push_opt!(props, "JoinsNamespaceOf", unit.joins_namespace_of);
        push_opt!(props, "RequiresMountsFor", unit.requires_mounts_for);
        push_opt!(props, "WantsMountsFor", unit.wants_mounts_for);
        push_opt!(props, "OnSuccessJobMode", unit.on_success_job_mode);
        push_opt!(props, "OnFailureJobMode", unit.on_failure_job_mode);
        push_opt!(props, "IgnoreOnIsolate", unit.ignore_on_isolate);
        push_opt!(props, "StopWhenUnneeded", unit.stop_when_unneded); // Note your struct typo "unneded"
        push_opt!(props, "RefuseManualStart", unit.refuse_manual_start);
        push_opt!(props, "RefuseManualStop", unit.refuse_manual_stop);
        push_opt!(props, "AllowIsolate", unit.allow_isolate);
        push_opt!(props, "DefaultDependencies", unit.default_dependencies);
        push_opt!(
            props,
            "SurviveFinalKillSignal",
            unit.survive_final_kill_signal
        );
        push_opt!(props, "CollectMode", unit.collect_mode);
        push_opt!(props, "FailureAction", unit.failure_action);
        push_opt!(props, "SuccessAction", unit.success_action);
        push_opt!(
            props,
            "FailureActionExitStatus",
            unit.failure_action_exit_status
        );
        push_opt!(
            props,
            "SuccessActionExitStatus",
            unit.success_action_exit_status
        );
        push_opt!(props, "JobTimeoutUSec", unit.job_timeout_usec);
        push_opt!(
            props,
            "JobRunningTimeoutUSec",
            unit.job_running_timeout_usec
        );
        push_opt!(props, "JobTimeoutAction", unit.job_timeout_action);
        push_opt!(
            props,
            "JobTimeoutRebootArgument",
            unit.job_timeout_reboot_argument
        );
        push_opt!(
            props,
            "StartLimitIntervalUSec",
            unit.start_limit_interval_usec
        );
        push_opt!(props, "StartLimitBurst", unit.start_limit_burst);
        push_opt!(props, "StartLimitAction", unit.start_limit_action);
        push_opt!(props, "RebootArgument", unit.reboot_argument);
        push_opt!(props, "SourcePath", unit.source_path);
        props
    }
}

#[derive(Builder)]
pub struct Resources<'a> {
    // CPU Control
    pub cpu_weight: Option<u64>,
    pub startup_cpu_weight: Option<u64>,
    pub cpu_quota_per_sec_usec: Option<u64>,
    pub cpu_quota_period_usec: Option<u64>,
    pub allowed_cpus: Option<&'a [u8]>,
    pub startup_allowed_cpus: Option<&'a [u8]>,
    // Memory Accounting and Control
    pub memory_accounting: Option<bool>,
    pub memory_min: Option<u64>,
    pub memory_low: Option<u64>,
    pub startup_memory_low: Option<u64>,
    pub default_memory_low: Option<u64>,
    pub default_memory_min: Option<u64>,
    pub default_startup_memory_low: Option<u64>,
    pub memory_max: Option<u64>,
    pub memory_high: Option<u64>,
    pub startup_memory_max: Option<u64>,
    pub startup_memory_high: Option<u64>,
    pub memory_swap_max: Option<u64>,
    pub startup_memory_swap_max: Option<u64>,
    pub memory_zswap_max: Option<u64>,
    pub startup_memory_zswap_max: Option<u64>,
    pub memory_zswap_writeback: Option<bool>,
    pub allowed_memory_nodes: Option<&'a [u8]>,
    pub startup_allowed_memory_nodes: Option<&'a [u8]>,
    // Process Accounting and Control
    pub tasks_accounting: Option<bool>,
    pub tasks_max: Option<u64>,
    // IO Accounting and Control
    pub io_accounting: Option<bool>,
    pub io_weight: Option<u64>,
    pub startup_io_weight: Option<u64>,
    pub io_device_weight: Option<&'a [(&'a str, u64)]>,
    pub io_read_bandwidth_max: Option<&'a [(&'a str, u64)]>,
    pub io_write_bandwidth_max: Option<&'a [(&'a str, u64)]>,
    pub io_read_iops_max: Option<&'a [(&'a str, u64)]>,
    pub io_write_iops_max: Option<&'a [(&'a str, u64)]>,
    pub io_device_latency_target_usec: Option<&'a [(&'a str, u64)]>,
    // Network Accounting and Control
    pub ip_accounting: Option<bool>,
    pub ip_address_allow: Option<&'a [(i32, &'a [u8], u32)]>,
    pub ip_address_deny: Option<&'a [(i32, &'a [u8], u32)]>,
    pub socket_bind_allow: Option<&'a [(i32, i32, u16, u16)]>,
    pub socket_bind_deny: Option<&'a [(i32, i32, u16, u16)]>,
    pub restrict_network_interfaces: Option<(bool, &'a [&'a str])>,
    pub bind_network_interface: Option<&'a str>,
    pub nft_set: Option<&'a [(i32, i32, &'a str, &'a str)]>,
    // BPF Programs
    pub ip_ingress_filter_path: Option<&'a [&'a str]>,
    pub ip_egress_filter_path: Option<&'a [&'a str]>,
    pub bpf_program: Option<&'a [(&'a str, &'a str)]>,
    // Device Access
    pub device_allow: Option<&'a [(&'a str, &'a str)]>,
    pub device_policy: Option<&'a str>,
    // Control Group Management
    pub slice: Option<&'a str>,
    pub delegate: Option<bool>,
    pub delegate_subgroup: Option<&'a str>,
    pub delegate_controllers: Option<&'a [&'a str]>,
    pub disable_controllers: Option<&'a [&'a str]>,
    // Memory Pressure Control
    pub managed_oom_swap: Option<&'a str>,
    pub managed_oom_memory_pressure: Option<&'a str>,
    pub managed_oom_memory_pressure_limit: Option<&'a str>,
    pub managed_oom_memory_pressure_duration_usec: Option<u64>,
    pub managed_oom_preference: Option<&'a str>,
    pub memory_pressure_watch: Option<&'a str>,
    pub memory_pressure_threshold_usec: Option<u64>,
    // Coredump Control
    pub coredump_receive: Option<bool>,
}

impl<'a> From<Resources<'a>> for Vec<(&'a str, Value<'a>)> {
    fn from(res: Resources<'a>) -> Self {
        let mut props = Vec::new();
        // CPU
        push_opt!(props, "CPUWeight", res.cpu_weight);
        push_opt!(props, "StartupCPUWeight", res.startup_cpu_weight);
        push_opt!(props, "CPUQuotaPerSecUSec", res.cpu_quota_per_sec_usec);
        push_opt!(props, "CPUQuotaPeriodUSec", res.cpu_quota_period_usec);
        push_opt!(props, "AllowedCPUs", res.allowed_cpus);
        push_opt!(props, "StartupAllowedCPUs", res.startup_allowed_cpus);
        // Memory
        push_opt!(props, "MemoryAccounting", res.memory_accounting);
        push_opt!(props, "MemoryMin", res.memory_min);
        push_opt!(props, "MemoryLow", res.memory_low);
        push_opt!(props, "StartupMemoryLow", res.startup_memory_low);
        push_opt!(props, "DefaultMemoryLow", res.default_memory_low);
        push_opt!(props, "DefaultMemoryMin", res.default_memory_min);
        push_opt!(
            props,
            "DefaultStartupMemoryLow",
            res.default_startup_memory_low
        );
        push_opt!(props, "MemoryMax", res.memory_max);
        push_opt!(props, "MemoryHigh", res.memory_high);
        push_opt!(props, "StartupMemoryMax", res.startup_memory_max);
        push_opt!(props, "StartupMemoryHigh", res.startup_memory_high);
        push_opt!(props, "MemorySwapMax", res.memory_swap_max);
        push_opt!(props, "StartupMemorySwapMax", res.startup_memory_swap_max);
        push_opt!(props, "MemoryZSwapMax", res.memory_zswap_max);
        push_opt!(props, "StartupMemoryZSwapMax", res.startup_memory_zswap_max);
        push_opt!(props, "MemoryZSwapWriteback", res.memory_zswap_writeback);
        push_opt!(props, "AllowedMemoryNodes", res.allowed_memory_nodes);
        push_opt!(
            props,
            "StartupAllowedMemoryNodes",
            res.startup_allowed_memory_nodes
        );
        // Tasks
        push_opt!(props, "TasksAccounting", res.tasks_accounting);
        push_opt!(props, "TasksMax", res.tasks_max);
        // IO
        push_opt!(props, "IOAccounting", res.io_accounting);
        push_opt!(props, "IOWeight", res.io_weight);
        push_opt!(props, "StartupIOWeight", res.startup_io_weight);
        push_opt!(props, "IODeviceWeight", res.io_device_weight);
        push_opt!(props, "IOReadBandwidthMax", res.io_read_bandwidth_max);
        push_opt!(props, "IOWriteBandwidthMax", res.io_write_bandwidth_max);
        push_opt!(props, "IOReadIOPSMax", res.io_read_iops_max);
        push_opt!(props, "IOWriteIOPSMax", res.io_write_iops_max);
        push_opt!(
            props,
            "IODeviceLatencyTargetUSec",
            res.io_device_latency_target_usec
        );
        // Network Accounting and Control
        push_opt!(props, "IPAccounting", res.ip_accounting);
        push_opt!(props, "IPAddressAllow", res.ip_address_allow);
        push_opt!(props, "IPAddressDeny", res.ip_address_deny);
        push_opt!(props, "SocketBindAllow", res.socket_bind_allow);
        push_opt!(props, "SocketBindDeny", res.socket_bind_deny);
        push_opt!(
            props,
            "RestrictNetworkInterfaces",
            res.restrict_network_interfaces
        );
        push_opt!(props, "NFTSet", res.nft_set);
        // BPF Programs
        push_opt!(props, "IPIngressFilterPath", res.ip_ingress_filter_path);
        push_opt!(props, "IPEgressFilterPath", res.ip_egress_filter_path);
        push_opt!(props, "BPFProgram", res.bpf_program);
        // Device Access
        push_opt!(props, "DeviceAllow", res.device_allow);
        push_opt!(props, "DevicePolicy", res.device_policy);
        // CGroup
        push_opt!(props, "Slice", res.slice);
        push_opt!(props, "Delegate", res.delegate);
        push_opt!(props, "DelegateSubgroup", res.delegate_subgroup);
        push_opt!(props, "DelegateControllers", res.delegate_controllers);
        push_opt!(props, "DisableControllers", res.disable_controllers);
        props
    }
}

#[derive(Builder)]
pub struct Exec<'a> {
    // Paths
    pub exec_search_path: Option<&'a [&'a str]>,
    pub working_directory: Option<&'a str>,
    pub root_directory: Option<&'a str>,
    pub root_image: Option<&'a str>,
    pub root_image_options: Option<&'a [(&'a str, &'a str)]>,
    pub root_ephemeral: Option<bool>,
    pub root_hash: Option<&'a [u8]>,
    pub root_hash_signature: Option<&'a [u8]>,
    pub root_verity: Option<&'a str>,
    pub root_image_policy: Option<&'a str>,
    pub mount_image_policy: Option<&'a str>,
    pub extensions_image_policy: Option<&'a str>,
    pub mount_apivfs: Option<bool>,
    pub bind_log_sockets: Option<bool>,
    pub protect_proc: Option<&'a str>,
    pub proc_subset: Option<&'a str>,
    pub bind_paths: Option<&'a [(&'a str, &'a str, bool, u64)]>,
    pub bind_read_only_paths: Option<&'a [(&'a str, &'a str, bool, u64)]>,
    pub mount_images: Option<&'a [(&'a str, &'a str, bool, &'a [(&'a str, &'a str)])]>,
    pub extension_images: Option<&'a [(&'a str, bool, &'a [(&'a str, &'a str)])]>,
    pub extension_directories: Option<&'a [&'a str]>,
    // User/Group Identity
    pub user: Option<&'a str>,
    pub group: Option<&'a str>,
    pub dynamic_user: Option<bool>,
    pub supplementary_groups: Option<&'a [&'a str]>,
    pub set_login_environment: Option<bool>,
    pub pam_name: Option<&'a str>,
    // Capabilities
    pub capability_bounding_set: Option<u64>,
    pub ambient_capabilities: Option<u64>,
    // Security
    pub no_new_privileges: Option<bool>,
    pub secure_bits: Option<i32>,
    // Mandatory Access Control
    pub se_linux_context: Option<(bool, &'a str)>,
    pub app_armor_profile: Option<(bool, &'a str)>,
    pub smack_process_label: Option<(bool, &'a str)>,
    // Process Properties
    pub limit_cpu: Option<u64>,
    pub limit_fsize: Option<u64>,
    pub limit_data: Option<u64>,
    pub limit_stack: Option<u64>,
    pub limit_core: Option<u64>,
    pub limit_rss: Option<u64>,
    pub limit_nofile: Option<u64>,
    pub limit_as: Option<u64>,
    pub limit_nproc: Option<u64>,
    pub limit_memlock: Option<u64>,
    pub limit_locks: Option<u64>,
    pub limit_sigpending: Option<u64>,
    pub limit_msgqueue: Option<u64>,
    pub limit_nice: Option<u64>,
    pub limit_rtprio: Option<u64>,
    pub limit_rttime: Option<u64>,
    pub umask: Option<u32>,
    pub coredump_filter: Option<u64>,
    pub keyring_mode: Option<&'a str>,
    pub oom_score_adjust: Option<i32>,
    pub timer_slack_nsec: Option<u64>,
    pub personality: Option<&'a str>,
    pub ignore_sigpipe: Option<bool>,
    // Scheduling
    pub nice: Option<i32>,
    pub cpu_scheduling_policy: Option<i32>,
    pub cpu_scheduling_priority: Option<i32>,
    pub cpu_scheduling_reset_on_fork: Option<bool>,
    pub cpu_affinity: Option<&'a [u8]>,
    pub numa_policy: Option<i32>,
    pub io_scheduling_class: Option<i32>,
    pub io_scheduling_priority: Option<i32>,
    // Sandboxing
    pub protecty_system: Option<&'a str>,
    pub protecty_home: Option<&'a str>,
    pub runtime_directory: Option<&'a [&'a str]>,
    pub state_directory: Option<&'a [&'a str]>,
    pub cache_directory: Option<&'a [&'a str]>,
    pub logs_directory: Option<&'a [&'a str]>,
    pub configuration_directory: Option<&'a [&'a str]>,
    pub runtime_directory_mode: Option<u32>,
    pub state_directory_mode: Option<u32>,
    pub cache_directory_mode: Option<u32>,
    pub logs_directory_mode: Option<u32>,
    pub configuration_directory_mode: Option<u32>,
    pub state_directory_quota: Option<(u64, u32, &'a str)>,
    pub cache_directory_quota: Option<(u64, u32, &'a str)>,
    pub logs_directory_quota: Option<(u64, u32, &'a str)>,
    pub runtime_directory_preserve: Option<&'a str>,
    pub timeout_clean_usec: Option<u64>,
    pub read_write_paths: Option<&'a [&'a str]>,
    pub read_only_paths: Option<&'a [&'a str]>,
    pub inaccessible_paths: Option<&'a [&'a str]>,
    pub exec_paths: Option<&'a [&'a str]>,
    pub no_exec_paths: Option<&'a [&'a str]>,
    pub temporary_file_system: Option<&'a [(&'a str, &'a str)]>,
    pub private_tmp: Option<bool>,
    pub private_tmp_ex: Option<&'a str>,
    pub private_devices: Option<bool>,
    pub private_network: Option<bool>,
    pub user_namespace_path: Option<&'a str>,
    pub network_namespace_path: Option<&'a str>,
    pub private_ipc: Option<bool>,
    pub ipc_namespace_path: Option<&'a str>,
    pub memory_ksm: Option<bool>,
    pub memory_thp: Option<&'a str>,
    pub private_pids: Option<&'a str>,
    pub private_users: Option<bool>,
    pub private_users_ex: Option<&'a str>,
    pub protect_hostname: Option<bool>,
    pub protect_hostname_ex: Option<(&'a str, &'a str)>,
    pub protect_clock: Option<bool>,
    pub protect_kernel_tunables: Option<bool>,
    pub protect_kernel_modules: Option<bool>,
    pub protect_kernel_logs: Option<bool>,
    pub protect_control_groups: Option<bool>,
    pub protect_control_groups_ex: Option<&'a str>,
    pub restrict_address_families: Option<(bool, &'a [&'a str])>,
    pub restrict_file_systems: Option<(bool, &'a [&'a str])>,
    pub restrict_namespaces: Option<u64>,
    pub delegate_namespaces: Option<u64>,
    pub private_bpf: Option<&'a str>,
    pub bpf_delegate_commands: Option<&'a str>,
    pub bpf_delegate_map: Option<&'a str>,
    pub bpf_delegate_programs: Option<&'a str>,
    pub bpf_delegate_attachments: Option<&'a str>,
    pub lock_personality: Option<bool>,
    pub memory_deny_write_execute: Option<bool>,
    pub restrict_realtime: Option<bool>,
    pub restrict_suidsgid: Option<bool>,
    pub remove_ipc: Option<bool>,
    pub private_mounts: Option<bool>,
    pub mount_flags: Option<u64>,
    // System Call Filtering
    pub system_call_filter: Option<(bool, &'a [&'a str])>,
    pub system_call_error_number: Option<i32>,
    pub system_call_architechtures: Option<&'a [&'a str]>,
    pub system_call_log: Option<(bool, &'a [&'a str])>,
    // Environment
    pub environment: Option<&'a [&'a str]>,
    pub environment_file: Option<&'a [(&'a str, bool)]>,
    pub pass_environment: Option<&'a [&'a str]>,
    pub unset_environment: Option<&'a [&'a str]>,
    // Logging and Standard Input/Output
    pub standard_input: Option<&'a str>,
    pub standard_output: Option<&'a str>,
    pub standard_error: Option<&'a str>,
    pub standard_input_data: Option<&'a [u8]>,
    pub log_level_max: Option<i32>,
    pub log_extra_fields: Option<&'a [&'a [u8]]>,
    pub log_rate_limit_interval_usec: Option<u64>,
    pub log_rate_limit_burst: Option<u32>,
    pub log_filter_patterns: Option<&'a [(bool, &'a str)]>,
    pub log_namespace: Option<&'a str>,
    pub syslog_identifier: Option<&'a str>,
    pub syslog_facility: Option<i32>,
    pub syslog_level: Option<i32>,
    pub syslog_level_prefix: Option<bool>,
    pub tty_path: Option<&'a str>,
    pub tty_reset: Option<bool>,
    pub tty_vhangup: Option<bool>,
    pub tty_columns: Option<u16>,
    pub tty_rows: Option<u16>,
    pub tty_vtdisallocate: Option<bool>,
    // Credentials
    pub load_credential: Option<&'a [(&'a str, &'a str)]>,
    pub load_credential_encrypted: Option<&'a [(&'a str, &'a str)]>,
    pub import_credential: Option<&'a [&'a str]>,
    pub import_credential_ex: Option<&'a [(&'a str, &'a str)]>,
    pub set_credential: Option<&'a [(&'a str, &'a [u8])]>,
    pub set_credential_encrypted: Option<&'a [(&'a str, &'a [u8])]>,
    // System V Compatibility
    pub utmp_identifier: Option<&'a str>,
    pub utmp_mode: Option<&'a str>,
}

impl<'a> From<Exec<'a>> for Vec<(&'a str, Value<'a>)> {
    fn from(exec: Exec<'a>) -> Self {
        let mut props = Vec::with_capacity(50); // High capacity to avoid reallocations
        // Paths
        push_opt!(props, "ExecSearchPath", exec.exec_search_path);
        push_opt!(props, "WorkingDirectory", exec.working_directory);
        push_opt!(props, "RootDirectory", exec.root_directory);
        push_opt!(props, "RootImage", exec.root_image);
        push_opt!(props, "RootImageOptions", exec.root_image_options);
        push_opt!(props, "RootEphemeral", exec.root_ephemeral);
        push_opt!(props, "RootHash", exec.root_hash);
        push_opt!(props, "RootHashSignature", exec.root_hash_signature);
        push_opt!(props, "RootVerity", exec.root_verity);
        push_opt!(props, "RootImagePolicy", exec.root_image_policy);
        push_opt!(props, "MountImagePolicy", exec.mount_image_policy);
        push_opt!(props, "ExtensionImagePolicy", exec.extensions_image_policy);
        push_opt!(props, "MountAPIVFS", exec.mount_apivfs);
        push_opt!(props, "BindLogSockets", exec.bind_log_sockets);
        push_opt!(props, "ProtectProc", exec.protect_proc);
        push_opt!(props, "ProcSubset", exec.proc_subset);
        push_opt!(props, "BindPaths", exec.bind_paths);
        push_opt!(props, "BindReadOnlyPaths", exec.bind_read_only_paths);
        push_opt!(props, "MountImages", exec.mount_images);
        push_opt!(props, "ExtensionImages", exec.extension_images);
        push_opt!(props, "ExtensionDirectories", exec.extension_directories);
        // User/Group Identity
        push_opt!(props, "User", exec.user);
        push_opt!(props, "Group", exec.group);
        push_opt!(props, "DynamicUser", exec.dynamic_user);
        push_opt!(props, "SupplementaryGroups", exec.supplementary_groups);
        push_opt!(props, "SetLoginEnvironment", exec.set_login_environment);
        push_opt!(props, "PAMName", exec.pam_name);
        // Capabilities
        push_opt!(props, "CapabilityBoundingSet", exec.capability_bounding_set);
        push_opt!(props, "AmbientCapabilities", exec.ambient_capabilities);
        // Security
        push_opt!(props, "NoNewPrivileges", exec.no_new_privileges);
        push_opt!(props, "SecureBits", exec.secure_bits);
        // Mandatory Access Control
        push_opt!(props, "SELinuxContext", exec.se_linux_context);
        push_opt!(props, "AppArmorProfile", exec.app_armor_profile);
        push_opt!(props, "SmackProcessLabel", exec.smack_process_label);
        // Process Properties
        push_opt!(props, "LimitCPU", exec.limit_cpu);
        push_opt!(props, "LimitFSIZE", exec.limit_fsize);
        push_opt!(props, "LimitDATA", exec.limit_data);
        push_opt!(props, "LimitSTACK", exec.limit_stack);
        push_opt!(props, "LimitCORE", exec.limit_core);
        push_opt!(props, "LimitRSS", exec.limit_rss);
        push_opt!(props, "LimitNOFILE", exec.limit_nofile);
        push_opt!(props, "LimitAS", exec.limit_as);
        push_opt!(props, "LimitNPROC", exec.limit_nproc);
        push_opt!(props, "LimitMEMLOCK", exec.limit_memlock);
        push_opt!(props, "LimitLOCKS", exec.limit_locks);
        push_opt!(props, "LimitSIGPENDING", exec.limit_sigpending);
        push_opt!(props, "LimitMSGQUEUE", exec.limit_msgqueue);
        push_opt!(props, "LimitNICE", exec.limit_nice);
        push_opt!(props, "LimitRTPRIO", exec.limit_rtprio);
        push_opt!(props, "LimitRTTIME", exec.limit_rttime);
        push_opt!(props, "UMask", exec.umask);
        push_opt!(props, "CoredumpFilter", exec.coredump_filter);
        push_opt!(props, "KeyringMode", exec.keyring_mode);
        push_opt!(props, "OOMScoreAdjust", exec.oom_score_adjust);
        push_opt!(props, "TimerSlackNSec", exec.timer_slack_nsec);
        push_opt!(props, "Personality", exec.personality);
        push_opt!(props, "IgnoreSIGPIPE", exec.ignore_sigpipe);
        // Scheduling
        push_opt!(props, "Nice", exec.nice);
        push_opt!(props, "CPUSchedulingPolicy", exec.cpu_scheduling_policy);
        push_opt!(props, "CPUSchedulingPriority", exec.cpu_scheduling_priority);
        push_opt!(
            props,
            "CPUSchedulingResetOnFork",
            exec.cpu_scheduling_reset_on_fork
        );
        push_opt!(props, "CPUAffinity", exec.cpu_affinity);
        push_opt!(props, "NUMAPolicy", exec.numa_policy);
        push_opt!(props, "IOSchedulingClass", exec.io_scheduling_class);
        push_opt!(props, "IOSchedulingPriority", exec.io_scheduling_priority);
        // Sandboxing
        push_opt!(props, "ProtectSystem", exec.protecty_system);
        push_opt!(props, "ProtectHome", exec.protecty_home);
        push_opt!(props, "RuntimeDirectory", exec.runtime_directory);
        push_opt!(props, "StateDirectory", exec.state_directory);
        push_opt!(props, "CacheDirectory", exec.cache_directory);
        push_opt!(props, "LogsDirectory", exec.logs_directory);
        push_opt!(
            props,
            "ConfigurationDirectory",
            exec.configuration_directory
        );
        push_opt!(props, "RuntimeDirectoryMode", exec.runtime_directory_mode);
        push_opt!(props, "StateDirectoryMode", exec.state_directory_mode);
        push_opt!(props, "CacheDirectoryMode", exec.cache_directory_mode);
        push_opt!(props, "LogsDirectoryMode", exec.logs_directory_mode);
        push_opt!(
            props,
            "ConfigurationDirectoryMode",
            exec.configuration_directory_mode
        );
        push_opt!(props, "StateDirectoryQuota", exec.state_directory_quota);
        push_opt!(props, "CacheDirectoryQuota", exec.cache_directory_quota);
        push_opt!(props, "LogsDirectoryQuota", exec.logs_directory_quota);
        push_opt!(
            props,
            "RuntimeDirectoryPreserve",
            exec.runtime_directory_preserve
        );
        push_opt!(props, "TimeoutCleanUSec", exec.timeout_clean_usec);
        push_opt!(props, "ReadWritePaths", exec.read_write_paths);
        push_opt!(props, "ReadOnlyPaths", exec.read_only_paths);
        push_opt!(props, "InaccessiblePaths", exec.inaccessible_paths);
        push_opt!(props, "ExecPaths", exec.exec_paths);
        push_opt!(props, "NoExecPaths", exec.no_exec_paths);
        push_opt!(props, "TemporaryFileSystem", exec.temporary_file_system);
        push_opt!(props, "PrivateTmp", exec.private_tmp);
        push_opt!(props, "PrivateTmpEx", exec.private_tmp_ex);
        push_opt!(props, "PrivateDevices", exec.private_devices);
        push_opt!(props, "PrivateNetwork", exec.private_network);
        push_opt!(props, "UserNamespacePath", exec.user_namespace_path);
        push_opt!(props, "NetworkNamespacePath", exec.network_namespace_path);
        push_opt!(props, "PrivateIPC", exec.private_ipc);
        push_opt!(props, "IPCNamespacePath", exec.ipc_namespace_path);
        push_opt!(props, "MemoryKSM", exec.memory_ksm);
        push_opt!(props, "MemoryTHP", exec.memory_thp);
        push_opt!(props, "PrivatePIDs", exec.private_pids);
        push_opt!(props, "PrivateUsers", exec.private_users);
        push_opt!(props, "PrivateUsersEx", exec.private_users_ex);
        push_opt!(props, "ProtectHostname", exec.protect_hostname);
        push_opt!(props, "ProtectHostnameEx", exec.protect_hostname_ex);
        push_opt!(props, "ProtectClock", exec.protect_clock);
        push_opt!(props, "ProtectKernelTunables", exec.protect_kernel_tunables);
        push_opt!(props, "ProtectKernelModules", exec.protect_kernel_modules);
        push_opt!(props, "ProtectKernelLogs", exec.protect_kernel_logs);
        push_opt!(props, "ProtectControlGroups", exec.protect_control_groups);
        push_opt!(
            props,
            "ProtectControlGroupsEx",
            exec.protect_control_groups_ex
        );
        push_opt!(
            props,
            "RestrictAddressFamilies",
            exec.restrict_address_families
        );
        push_opt!(props, "RestrictFileSystems", exec.restrict_file_systems);
        push_opt!(
            props,
            "MemoryDenyWriteExecute",
            exec.memory_deny_write_execute
        );
        push_opt!(props, "RestrictRealtime", exec.restrict_realtime);
        push_opt!(props, "RestrictNamespaces", exec.restrict_namespaces);
        push_opt!(props, "RestrictSUIDSGID", exec.restrict_suidsgid);
        push_opt!(props, "RemoveIPC", exec.remove_ipc);
        push_opt!(props, "PrivateMounts", exec.private_mounts);
        push_opt!(props, "MountFlags", exec.mount_flags);
        push_opt!(props, "DelegateNamespaces", exec.delegate_namespaces);
        push_opt!(props, "PrivateBPF", exec.private_bpf);
        push_opt!(props, "BPFDelegateCommands", exec.bpf_delegate_commands);
        push_opt!(props, "BPFDelegateMap", exec.bpf_delegate_map);
        push_opt!(props, "BPFDelegatePrograms", exec.bpf_delegate_programs);
        push_opt!(
            props,
            "BPFDelegateAttachments",
            exec.bpf_delegate_attachments
        );
        push_opt!(props, "LockPersonality", exec.lock_personality);
        // System Call Filtering
        push_opt!(props, "SystemCallFilter", exec.system_call_filter);
        push_opt!(
            props,
            "SystemCallErrorNumber",
            exec.system_call_error_number
        );
        push_opt!(
            props,
            "SystemCallArchitectures",
            exec.system_call_architechtures
        );
        push_opt!(props, "SystemCallLog", exec.system_call_log);
        // Environment
        push_opt!(props, "Environment", exec.environment);
        push_opt!(props, "EnvironmentFile", exec.environment_file);
        push_opt!(props, "PassEnvironment", exec.pass_environment);
        push_opt!(props, "UnsetEnvironment", exec.unset_environment);
        // Logging / Standard I/O
        push_opt!(props, "StandardInput", exec.standard_input);
        push_opt!(props, "StandardOutput", exec.standard_output);
        push_opt!(props, "StandardError", exec.standard_error);
        push_opt!(props, "StandardInputData", exec.standard_input_data);
        push_opt!(props, "LogLevelMax", exec.log_level_max);
        push_opt!(props, "LogExtraFields", exec.log_extra_fields);
        push_opt!(
            props,
            "LogRateLimitIntervalUSec",
            exec.log_rate_limit_interval_usec
        );
        push_opt!(props, "LogRateLimitBurst", exec.log_rate_limit_burst);
        push_opt!(props, "LogFilterPatterns", exec.log_filter_patterns);
        push_opt!(props, "LogNamespace", exec.log_namespace);
        push_opt!(props, "SyslogIdentifier", exec.syslog_identifier);
        push_opt!(props, "SyslogFacility", exec.syslog_facility);
        push_opt!(props, "SyslogLevel", exec.syslog_level);
        push_opt!(props, "SyslogLevelPrefix", exec.syslog_level_prefix);
        push_opt!(props, "TTYPath", exec.tty_path);
        push_opt!(props, "TTYReset", exec.tty_reset);
        push_opt!(props, "TTYVHangup", exec.tty_vhangup);
        push_opt!(props, "TTYColumns", exec.tty_columns);
        push_opt!(props, "TTYRows", exec.tty_rows);
        push_opt!(props, "TTYVTDisallocate", exec.tty_vtdisallocate);
        // Credentials
        push_opt!(props, "LoadCredential", exec.load_credential);
        push_opt!(
            props,
            "LoadCredentialEncrypted",
            exec.load_credential_encrypted
        );
        push_opt!(props, "ImportCredential", exec.import_credential);
        push_opt!(props, "ImportCredentialEx", exec.import_credential_ex);
        push_opt!(props, "SetCredential", exec.set_credential);
        push_opt!(
            props,
            "SetCredentialEncrypted",
            exec.set_credential_encrypted
        );
        // System V
        push_opt!(props, "UtmpIdentifier", exec.utmp_identifier);
        push_opt!(props, "UtmpMode", exec.utmp_mode);
        props
    }
}

#[derive(Builder)]
pub struct Kill<'a> {
    pub kill_mode: Option<&'a str>,
    pub kill_signal: Option<i32>,
    pub restart_kill_signal: Option<i32>,
    pub send_sighup: Option<bool>,
    pub send_sigkill: Option<bool>,
    pub final_kill_signal: Option<i32>,
    pub watchdog_signal: Option<i32>,
}

impl<'a> From<Kill<'a>> for Vec<(&'a str, Value<'a>)> {
    fn from(kill: Kill<'a>) -> Self {
        let mut props = Vec::new();
        push_opt!(props, "KillMode", kill.kill_mode);
        push_opt!(props, "KillSignal", kill.kill_signal);
        push_opt!(props, "RestartKillSignal", kill.restart_kill_signal);
        push_opt!(props, "SendSIGHUP", kill.send_sighup);
        push_opt!(props, "SendSIGKILL", kill.send_sigkill);
        push_opt!(props, "FinalKillSignal", kill.final_kill_signal);
        push_opt!(props, "WatchdogSignal", kill.watchdog_signal);
        props
    }
}

#[derive(Builder)]
pub struct Service<'a> {
    pub unit: Option<Unit<'a>>,
    pub resources: Option<Resources<'a>>,
    pub exec: Option<Exec<'a>>,
    pub kill: Option<Kill<'a>>,
    // Service specific
    pub type_: Option<&'a str>,
    pub exit_type: Option<&'a str>,
    pub remain_after_exit: Option<bool>,
    pub guess_main_pid: Option<bool>,
    pub pid_file: Option<&'a str>,
    pub bus_name: Option<&'a str>,
    pub exec_start: &'a [(&'a str, &'a [&'a str], bool)],
    pub exec_start_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub exec_start_pre: Option<&'a [(&'a str, &'a [&'a str], bool)]>,
    pub exec_start_pre_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub exec_start_post: Option<&'a [(&'a str, &'a [&'a str], bool)]>,
    pub exec_start_post_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub exec_condition: Option<&'a [(&'a str, &'a [&'a str], bool)]>,
    pub exec_condition_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub exec_reload: Option<&'a [(&'a str, &'a [&'a str], bool)]>,
    pub exec_reload_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub exec_reload_post: Option<&'a [(&'a str, &'a [&'a str], bool)]>,
    pub exec_reload_post_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub exec_stop: Option<&'a [(&'a str, &'a [&'a str], bool)]>,
    pub exec_stop_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub exec_stop_post: Option<&'a [(&'a str, &'a [&'a str], bool)]>,
    pub exec_stop_post_ex: Option<&'a [(&'a str, &'a [&'a str], &'a [&'a str])]>,
    pub restart_usec: Option<u64>,
    pub restart_steps: Option<u32>,
    pub restart_max_delay_usec: Option<u64>,
    pub timeout_start_usec: Option<u64>,
    pub timeout_stop_usec: Option<u64>,
    pub timeout_abort_usec: Option<u64>,
    pub timeout_usec: Option<u64>,
    pub timeout_start_failure_mode: Option<&'a str>,
    pub timeout_stop_failure_mode: Option<&'a str>,
    pub runtime_max_usec: Option<u64>,
    pub runtime_randomized_extra_usec: Option<u64>,
    pub watchdog_usec: Option<u64>,
    pub restart: Option<&'a str>,
    pub restart_mode: Option<&'a str>,
    pub success_exit_status: Option<(&'a [i32], &'a [i32])>,
    pub restart_prevent_exit_status: Option<(&'a [i32], &'a [i32])>,
    pub restart_force_exist_status: Option<(&'a [i32], &'a [i32])>,
    pub root_directory_start_only: Option<bool>,
    pub non_blocking: Option<bool>,
    pub notify_access: Option<&'a str>,
    pub file_descriptor_store_max: Option<u32>,
    pub file_descriptor_store_preserve: Option<&'a str>,
    pub usb_function_descriptors: Option<&'a str>,
    pub usb_function_strings: Option<&'a str>,
    pub oom_policy: Option<&'a str>,
    pub open_file: Option<&'a [(&'a str, &'a str, u64)]>,
    pub reload_signal: Option<i32>,
}

impl<'a> From<Service<'a>> for Vec<(&'a str, Value<'a>)> {
    fn from(svc: Service<'a>) -> Self {
        let mut props = Vec::with_capacity(100);
        extend_opt!(props, svc.unit);
        extend_opt!(props, svc.resources);
        extend_opt!(props, svc.exec);
        extend_opt!(props, svc.kill);
        // Service-Specific Core Properties
        push_opt!(props, "Type", svc.type_);
        push_opt!(props, "ExitType", svc.exit_type);
        push_opt!(props, "RemainAfterExit", svc.remain_after_exit);
        push_opt!(props, "GuessMainPID", svc.guess_main_pid);
        push_opt!(props, "PIDFile", svc.pid_file);
        push_opt!(props, "BusName", svc.bus_name);
        // Execution Commands
        props.push(("ExecStart", Value::from(svc.exec_start)));
        push_opt!(props, "ExecStartEx", svc.exec_start_ex);
        push_opt!(props, "ExecStartPre", svc.exec_start_pre);
        push_opt!(props, "ExecStartPreEx", svc.exec_start_pre_ex);
        push_opt!(props, "ExecStartPost", svc.exec_start_post);
        push_opt!(props, "ExecStartPostEx", svc.exec_start_post_ex);
        push_opt!(props, "ExecCondition", svc.exec_condition);
        push_opt!(props, "ExecConditionEx", svc.exec_condition_ex);
        push_opt!(props, "ExecReload", svc.exec_reload);
        push_opt!(props, "ExecReloadEx", svc.exec_reload_ex);
        push_opt!(props, "ExecReloadPost", svc.exec_reload_post);
        push_opt!(props, "ExecReloadPostEx", svc.exec_reload_post_ex);
        push_opt!(props, "ExecStop", svc.exec_stop);
        push_opt!(props, "ExecStopEx", svc.exec_stop_ex);
        push_opt!(props, "ExecStopPost", svc.exec_stop_post);
        push_opt!(props, "ExecStopPostEx", svc.exec_stop_post_ex);
        // Timeouts & Restarts
        push_opt!(props, "RestartUSec", svc.restart_usec);
        push_opt!(props, "RestartSteps", svc.restart_steps);
        push_opt!(props, "RestartMaxDelayUSec", svc.restart_max_delay_usec);
        push_opt!(props, "TimeoutStartUSec", svc.timeout_start_usec);
        push_opt!(props, "TimeoutStopUSec", svc.timeout_stop_usec);
        push_opt!(props, "TimeoutAbortUSec", svc.timeout_abort_usec);
        push_opt!(props, "TimeoutUSec", svc.timeout_usec);
        push_opt!(
            props,
            "TimeoutStartFailureMode",
            svc.timeout_start_failure_mode
        );
        push_opt!(
            props,
            "TimeoutStopFailureMode",
            svc.timeout_stop_failure_mode
        );
        push_opt!(props, "RuntimeMaxUSec", svc.runtime_max_usec);
        push_opt!(
            props,
            "RuntimeRandomizedExtraUSec",
            svc.runtime_randomized_extra_usec
        );
        push_opt!(props, "WatchdogUSec", svc.watchdog_usec);
        push_opt!(props, "Restart", svc.restart);
        push_opt!(props, "RestartMode", svc.restart_mode);
        // Exit Status and Policies
        push_opt!(props, "SuccessExitStatus", svc.success_exit_status);
        push_opt!(
            props,
            "RestartPreventExitStatus",
            svc.restart_prevent_exit_status
        );
        push_opt!(
            props,
            "RestartForceExitStatus",
            svc.restart_force_exist_status
        );
        push_opt!(
            props,
            "RootDirectoryStartOnly",
            svc.root_directory_start_only
        );
        push_opt!(props, "NonBlocking", svc.non_blocking);
        push_opt!(props, "NotifyAccess", svc.notify_access);
        push_opt!(
            props,
            "FileDescriptorStoreMax",
            svc.file_descriptor_store_max
        );
        push_opt!(
            props,
            "FileDescriptorStorePreserve",
            svc.file_descriptor_store_preserve
        );
        push_opt!(
            props,
            "USBFunctionDescriptors",
            svc.usb_function_descriptors
        );
        push_opt!(props, "USBFunctionStrings", svc.usb_function_strings);
        push_opt!(props, "OOMPolicy", svc.oom_policy);
        push_opt!(props, "OpenFile", svc.open_file);
        push_opt!(props, "ReloadSignal", svc.reload_signal);
        props
    }
}

#[derive(Builder)]
pub struct Scope<'a> {
    pub unit: Option<Unit<'a>>,
    pub resources: Option<Resources<'a>>,
    pub kill: Option<Kill<'a>>,
    // Scope specific
    pub pidfds: Box<[Fd<'a>]>,
    pub timeout_stop_usec: Option<u64>,
    pub oom_policy: Option<&'a str>,
    pub runtime_max_usec: Option<u64>,
    pub runtime_randomized_extra_usec: Option<u64>,
}

impl<'a> From<Scope<'a>> for Vec<(&'a str, Value<'a>)> {
    fn from(scope: Scope<'a>) -> Self {
        let mut props = Vec::new();
        extend_opt!(props, scope.unit);
        extend_opt!(props, scope.resources);
        extend_opt!(props, scope.kill);
        // Add Scope-specific properties
        props.push(("PIDFDs", Value::from(scope.pidfds.into_vec())));
        push_opt!(props, "TimeoutStopUSec", scope.timeout_stop_usec);
        push_opt!(props, "OOMPolicy", scope.oom_policy);
        push_opt!(props, "RuntimeMaxUSec", scope.runtime_max_usec);
        push_opt!(
            props,
            "RuntimeRandomizedExtraUSec",
            scope.runtime_randomized_extra_usec
        );
        props
    }
}

mod private {
    pub trait Sealed {}
    impl<'a> Sealed for crate::properties::Scope<'a> {}
    impl<'a> Sealed for crate::properties::Service<'a> {}
}

pub trait ToProperties<'a>:
    private::Sealed + Into<Vec<(&'a str, zbus::zvariant::Value<'a>)>>
{
}
impl<'a> ToProperties<'a> for crate::properties::Scope<'a> {}
impl<'a> ToProperties<'a> for crate::properties::Service<'a> {}
