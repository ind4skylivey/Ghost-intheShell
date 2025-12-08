/// Advanced security module for Ghost Shell
/// Provides memory protection, anti-forensics, and monitoring detection
use std::io;

#[cfg(target_os = "linux")]
use libc::{c_void, madvise, mlock, MADV_DONTDUMP};
#[cfg(target_os = "linux")]
use std::fs;

/// Security status of the shell
#[derive(Debug, Clone)]
pub struct SecurityStatus {
    pub memory_locked: bool,
    pub swap_disabled: bool,
    pub core_dumps_disabled: bool,
    pub monitoring_detected: bool,
    pub threats_detected: Vec<String>,
}

impl SecurityStatus {
    pub fn new() -> Self {
        SecurityStatus {
            memory_locked: false,
            swap_disabled: false,
            core_dumps_disabled: false,
            monitoring_detected: false,
            threats_detected: Vec::new(),
        }
    }

    /// Generate a status report string
    pub fn report(&self) -> String {
        let mut report = String::from("=== GHOST SHELL SECURITY STATUS ===\r\n");

        report.push_str(&format!(
            "Memory Locked:       {}\r\n",
            if self.memory_locked {
                "✓ YES"
            } else {
                "✗ NO"
            }
        ));

        report.push_str(&format!(
            "Swap Disabled:       {}\r\n",
            if self.swap_disabled {
                "✓ YES"
            } else {
                "⚠ NO (RISK: Memory may be swapped to disk)"
            }
        ));

        report.push_str(&format!(
            "Core Dumps Blocked:  {}\r\n",
            if self.core_dumps_disabled {
                "✓ YES"
            } else {
                "✗ NO"
            }
        ));

        report.push_str(&format!(
            "Monitoring Detected: {}\r\n",
            if self.monitoring_detected {
                "⚠ YES (Potential surveillance)"
            } else {
                "✓ NO"
            }
        ));

        if !self.threats_detected.is_empty() {
            report.push_str("\r\n⚠ THREATS DETECTED:\r\n");
            for threat in &self.threats_detected {
                report.push_str(&format!("  - {}\r\n", threat));
            }
        }

        report.push_str("\r\n");
        report
    }
}

/// Lock memory pages to prevent swapping to disk
#[allow(dead_code)]
#[cfg(target_os = "linux")]
pub fn lock_memory(ptr: *const u8, len: usize) -> io::Result<()> {
    unsafe {
        if mlock(ptr as *const c_void, len) == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn lock_memory(_ptr: *const u8, _len: usize) -> io::Result<()> {
    // Not supported on non-Linux platforms
    Ok(())
}

/// Prevent memory region from being included in core dumps
#[allow(dead_code)]
#[cfg(target_os = "linux")]
pub fn disable_core_dump(ptr: *const u8, len: usize) -> io::Result<()> {
    unsafe {
        if madvise(ptr as *mut c_void, len, MADV_DONTDUMP) == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn disable_core_dump(_ptr: *const u8, _len: usize) -> io::Result<()> {
    Ok(())
}

/// Check if swap is enabled on the system
#[cfg(target_os = "linux")]
pub fn is_swap_enabled() -> bool {
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("SwapTotal:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    if let Ok(swap_kb) = value.parse::<u64>() {
                        return swap_kb > 0;
                    }
                }
            }
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
pub fn is_swap_enabled() -> bool {
    false
}

/// Detect if we're being traced/monitored
#[cfg(target_os = "linux")]
pub fn detect_monitoring() -> Vec<String> {
    let mut threats = Vec::new();

    // Check if we're being ptraced
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("TracerPid:") {
                if let Some(pid) = line.split_whitespace().nth(1) {
                    if pid != "0" {
                        threats.push(format!("ptrace detected (PID: {})", pid));
                    }
                }
            }
        }
    }

    // Check for common monitoring tools
    let monitoring_tools = vec![
        "strace",
        "ltrace",
        "gdb",
        "auditd",
        "sysdig",
        "bpftrace",
        "perf",
        "systemtap",
    ];

    if let Ok(processes) = fs::read_dir("/proc") {
        for entry in processes.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.chars().all(|c| c.is_ascii_digit()) {
                    let cmdline_path = format!("/proc/{}/cmdline", file_name);
                    if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
                        for tool in &monitoring_tools {
                            if cmdline.contains(tool) {
                                threats.push(format!("Monitoring tool detected: {}", tool));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    threats
}

#[cfg(not(target_os = "linux"))]
pub fn detect_monitoring() -> Vec<String> {
    Vec::new()
}

/// Initialize security measures
pub fn initialize_security() -> SecurityStatus {
    let mut status = SecurityStatus::new();

    // Check swap
    status.swap_disabled = !is_swap_enabled();

    // Detect monitoring
    let threats = detect_monitoring();
    status.monitoring_detected = !threats.is_empty();
    status.threats_detected = threats;

    status
}

/// Anti-debugging: Check if debugger is attached
#[cfg(target_os = "linux")]
pub fn is_debugger_present() -> bool {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("TracerPid:") {
                if let Some(pid) = line.split_whitespace().nth(1) {
                    return pid != "0";
                }
            }
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
pub fn is_debugger_present() -> bool {
    false
}
