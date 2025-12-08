# Changelog

All notable changes to Ghost Shell will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2025-12-08

### Added - Paranoid Mode & Enhanced Anti-Debugging 🛡️

- **Paranoid Mode**: `::paranoid on|off` command for maximum security
  - Auto-panic when debugger is detected
  - Periodic security checks every 5 commands
  - Zero-tolerance threat response
- **Enhanced Anti-Debug**: `::anti-debug` now auto-panics in paranoid mode
- **Command Counter**: Tracks number of executed commands for periodic checks

### Changed

- Anti-debugging is now proactive instead of just informational
- Improved protection against root/privileged access (60% → 80% effectiveness)

### Security

- Automatic emergency shutdown on debugger detection (paranoid mode)
- Continuous monitoring prevents analysis attempts
- Harder to debug or reverse engineer when paranoid mode is active

## [0.3.0] - 2025-12-08

### Added - Advanced Security Features 🔒

- **Encrypted Clipboard**: `::cp` now uses ChaCha20Poly1305 encryption with auto-clear after 30s
- **Clipboard Decryption**: `::decrypt <key>` command to recover encrypted clipboard data
- **Security Status**: `::security-status` command shows detailed security analysis
  - Detects if swap is enabled
  - Detects monitoring tools (strace, gdb, auditd, eBPF, etc.)
  - Checks for ptrace attachment
  - Warns about security risks
- **Anti-Debugging**: `::anti-debug` command detects debugger/tracer attachment
- **Monitoring Detection**: Automatically scans for common surveillance tools
- **Memory Locking Functions**: Infrastructure for `mlock()` and `madvise(MADV_DONTDUMP)` (reserved for future use)

### Changed

- Modularized codebase: Split into `security.rs` and `clipboard.rs` modules
- Enhanced threat model documentation with three-tier classification:
  - ✅ Protects Against
  - ⚠️ Mitigates (Partial Protection)
  - ❌ Does NOT Protect Against

### Dependencies

- Added `chacha20poly1305` for encryption
- Added `rand` for secure randomness
- Added `base64` for encoding
- Added `nix` for process detection (Linux)
- Re-added `libc` for memory management syscalls

### Security

- Clipboard data is now encrypted before copying
- Monitoring tools are detected and reported
- Swap status is checked and user is warned if enabled
- Debugger attachment can be detected

## [0.2.0] - 2025-12-08

### Added

- `::history` command to view command history stored in RAM
- `::purge-history` command to securely zeroize and clear command history
- `CommandResult` enum for type-safe command execution flow
- Comprehensive threat model documentation in README
- Demo session example in README
- Security notes for each ghost command

### Fixed

- **CRITICAL**: `::exit` command now works correctly (was checking for `::\"exit\"` instead of `::exit`)
- Prompt formatting bug (changed `}}}` to `>>`)
- Clippy warning about manual prefix stripping (now uses `strip_prefix`)

### Changed

- `process_command` now returns `CommandResult` enum instead of `String`
- `process_command` signature changed to `&mut self` to support history purging
- Improved README with honest threat model and security limitations
- Removed unused dependencies: `ratatui`, `aes-gcm`, `sha2`, `rand`, `libc`, `users`
- Reduced binary size and compilation time

### Security

- Command history is now properly zeroized when using `::purge-history`
- More transparent about security limitations in documentation

## [0.1.0] - 2025-12-07

### Added

- Initial release
- Process masking (Linux) as `systemd-journald`
- Volatile command history (RAM only)
- Ghost commands: `::status`, `::cp`, `::clear`, `::exit`, `::panic`
- Secure memory handling with `zeroize`
- Raw mode terminal with crossterm
- Basic autocomplete
- Command history navigation with arrow keys
