<img width="1536" height="1024" alt="ghost" src="https://github.com/user-attachments/assets/c6bc43c9-2889-475c-b29f-728263452a5b" />

<div align="center">

# 👻 Ghost Shell (gsh)

[![Version](https://img.shields.io/badge/version-0.3.2-red?style=for-the-badge&logo=rust&logoColor=white)](https://github.com/ind4skylivey/Ghost-intheShell)
[![Security Audit](https://img.shields.io/badge/Security_Audit-PASSED-brightgreen?style=for-the-badge&logo=shield&logoColor=white)](./SECURITY_AUDIT.md)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](./LICENSE)

---

**🔴 RED TEAM TOOLS 🔴**

[![Anti-Forensics](https://img.shields.io/badge/Anti--Forensics-Enabled-critical?style=flat-square&logo=ghost&logoColor=white)](.)
[![Memory Safe](https://img.shields.io/badge/Memory-Zeroized-success?style=flat-square&logo=shield&logoColor=white)](.)
[![Process Masking](https://img.shields.io/badge/Process-Masked-orange?style=flat-square&logo=linux&logoColor=white)](.)
[![Encrypted Clipboard](https://img.shields.io/badge/Clipboard-ChaCha20-blueviolet?style=flat-square&logo=keybase&logoColor=white)](.)
[![Anti-Debug](https://img.shields.io/badge/Anti--Debug-Active-red?style=flat-square&logo=bug&logoColor=white)](.)
[![Paranoid Mode](https://img.shields.io/badge/Paranoid_Mode-Available-darkred?style=flat-square&logo=fire&logoColor=white)](.)

---

</div>

**Ghost Shell** is a secure, stealthy shell implementation in Rust designed for privacy and low-profile operations. It features process masking, secure memory handling, and built-in "ghost" commands for covert utilities.

> ⚠️ **Educational Tool**: This project is designed for security research, red-team exercises, and understanding shell internals. See [Threat Model](#-threat-model) below.

## 🛡️ Key Features

- **Process Masking (Linux):** Automatically disguises the process name as `systemd-journald` upon initialization to blend in with system processes.
- **Secure Memory:** Utilizes the `zeroize` crate to ensure input buffers and sensitive data are scrubbed from memory when dropped.
- **Volatile History:** Command history is kept strictly in RAM and is never written to disk (`.bash_history` etc.), ensuring no forensic trace remains after exit.
- **Ghost Commands (`::`):** A set of internal, prefixed commands that never touch the underlying system shell history.
- **Clipboard Injection:** Securely copy text to the system clipboard directly from the shell without trace files.
- **Dynamic Prompt:** Displays your current directory context `gsh <dir>>>` while keeping a low profile.

## 🚀 Installation

### Prerequisites

- Rust and Cargo (latest stable version)
- Linux environment (recommended for full feature support like process masking)
- System dependencies for clipboard support (e.g., `libxcb`, `libx11` on Linux might be required by `arboard`)

### Build from Source

```bash
git clone git@github.com:ind4skylivey/Ghost-intheShell.git
cd ghost-shell
cargo build --release
```

## 💻 Usage

Run the shell:

```bash
cargo run --release
# or directly execute the binary
./target/release/ghost-shell
```

### Demo Session

```bash
$ ./target/release/ghost-shell
Initializing Ghost Shell protocol...
gsh ghost-shell>> ::status
GHOST MODE ACTIVE. MEMORY SECURE. TRACE: NONE.

gsh ghost-shell>> ::cp my-super-secret-token-12345
DATA INJECTED TO CLIPBOARD. TRACES REMOVED.

gsh ghost-shell>> ls -la
total 48
drwxr-xr-x 6 user user  4096 Dec  8 03:45 .
drwxr-xr-x 3 user user  4096 Dec  8 01:30 ..
...

gsh ghost-shell>> ::history
Command History (RAM only):
  1: ::status
  2: ::cp my-super-secret-token-12345
  3: ls -la

gsh ghost-shell>> ::purge-history
HISTORY PURGED. 3 COMMANDS ZEROIZED FROM MEMORY.

gsh ghost-shell>> ::exit
[!] INITIATING SECURE SHUTDOWN...
[*] Overwriting memory buffers... DONE.
[*] All systems clear. Ghost Shell terminated.
```

### Navigation & UX

- **CD:** Native support for `cd` to change directories (e.g., `cd /tmp`, `cd ..`, `cd ~`).
- **Cursor:** Use `←` / `→` arrows to edit your command line.
- **History:** Use `↑` / `↓` arrows to cycle through previous commands (RAM only).
- **Autocomplete:** Press `Tab` to auto-complete filenames in the current directory.
- **Clear:** `Ctrl+L` or `clear` to clean the screen.

### 👻 Ghost Commands

Ghost commands are special instructions processed internally by the shell. They are prefixed with `::`.

| Command              | Description                                                                        | Security Notes                               |
| :------------------- | :--------------------------------------------------------------------------------- | :------------------------------------------- |
| `::status`           | Displays the current security status of the shell.                                 | Informational only                           |
| `::security-status`  | **Advanced:** Shows detailed security analysis (swap, monitoring, etc.)            | Detects threats                              |
| `::history`          | Shows command history stored in RAM.                                               | Reveals what you've typed this session       |
| `::purge-history`    | **Securely wipes** all command history from memory.                                | Zeroizes strings before clearing             |
| `::cp <text>`        | **Encrypted Copy:** Copies `<text>` to clipboard with ChaCha20Poly1305 encryption. | Auto-clears in 30s, returns decryption key   |
| `::decrypt <key>`    | Decrypts encrypted clipboard content using the provided key.                       | Requires key from `::cp` output              |
| `::anti-debug`       | Checks if a debugger/tracer is attached to the process.                            | Detects ptrace, auto-panics in paranoid mode |
| `::paranoid on\|off` | **Paranoid Mode:** Auto-panic on debugger + periodic checks every 5 commands.      | Maximum security, zero tolerance             |
| `::clear`            | Clears the terminal screen securely.                                               | Visual only, doesn't affect memory           |
| `::exit`             | Terminates the Ghost Shell session.                                                | Triggers secure shutdown                     |
| `::panic`            | **NUCLEAR OPTION:** Simulates a crash, wipes memory, and exits immediately.        | Emergency exit with fake kernel panic        |

**Example - Encrypted Clipboard:**

```bash
gsh ~/secrets>> ::cp my-super-secret-password-123
ENCRYPTED DATA INJECTED. KEY: a3F5dGhpcyBpcyBhIHJhbmRvbSBrZXk=
AUTO-CLEAR IN 30s.
Use ::decrypt to recover.

# Later, to decrypt:
gsh ~/secrets>> ::decrypt a3F5dGhpcyBpcyBhIHJhbmRvbSBrZXk=
Decrypted: my-super-secret-password-123
```

**Example - Security Status:**

```bash
gsh ~/secrets>> ::security-status
=== GHOST SHELL SECURITY STATUS ===
Memory Locked:       ✗ NO
Swap Disabled:       ⚠ NO (RISK: Memory may be swapped to disk)
Core Dumps Blocked:  ✗ NO
Monitoring Detected: ✓ NO
```

**Example - Paranoid Mode:**

```bash
gsh ~/secrets>> ::paranoid on
⚠ PARANOID MODE ENABLED
- Auto-panic on debugger detection
- Periodic security checks every 5 commands
- Enhanced threat monitoring

gsh ~/secrets>> ::anti-debug
✓ No debugger detected.

# If a debugger attaches:
gsh ~/secrets>> ls
⚠ PERIODIC CHECK: DEBUGGER DETECTED
PARANOID MODE - INITIATING EMERGENCY SHUTDOWN...
[Process exits with code 137]
```

## 🎯 Threat Model

### What Ghost Shell Protects Against ✅

- **Disk-based history forensics**: No `.bash_history`, `.zsh_history`, or similar files are created.
- **Casual process inspection**: Process name appears as `systemd-journald` in `ps`, `top`, etc.
- **Accidental command logging**: Ghost commands (`::`) never touch the system shell.
- **Memory residue (limited)**: Sensitive buffers are zeroized on drop.
- **Clipboard snooping (mitigated)**: Clipboard data is encrypted with ChaCha20Poly1305 and auto-cleared after 30s.
- **Monitoring detection**: Detects `ptrace`, `strace`, `gdb`, `auditd`, and other common monitoring tools.
- **Debugger attachment**: `::anti-debug` command detects if the process is being traced.

### What Ghost Shell Mitigates (Partial Protection) ⚠️

- **Swap files**: Detects if swap is enabled and warns user. Memory locking functions available for future use.
- **Core dumps**: Functions to exclude memory from core dumps (via `madvise`) are implemented but not yet active by default.
- **Clipboard monitoring**: While clipboard is encrypted, the key is displayed on screen. Use carefully.

### What Ghost Shell Does NOT Protect Against ❌

- **Root/privileged access**: Root can inspect `/proc/<pid>/exe`, memory dumps, etc.
- **Memory forensics (advanced)**: RAM dumps can still reveal command history before zeroization.
- **Swap files (if enabled)**: The OS may have swapped memory pages to disk before detection.
- **Screen recording/keyloggers**: If your terminal is being recorded, all commands are visible.
- **Advanced process hiding**: Only the process _name_ is masked; `/proc/<pid>/cmdline`, parent PID, and binary path are still visible.
- **Kernel-level monitoring (sophisticated)**: Custom kernel modules or eBPF programs can bypass user-space detection.

### Recommended Use Cases

- **Security research & education**: Understanding shell internals and memory management.
- **Red-team exercises**: Practicing operational security in controlled environments.
- **Privacy-conscious workflows**: Avoiding accidental command history leaks.
- **Malware analysis labs**: Isolated environments where you want minimal traces.

## 🔧 Technical Details

### Stack

- **Rust 2021 Edition**
- **crossterm**: Terminal manipulation and raw mode
- **zeroize**: Secure memory scrubbing
- **arboard**: Cross-platform clipboard access
- **chacha20poly1305**: Authenticated encryption
- **prctl** (Linux): Process name masking

### Architecture

- **Modular implementation**: `main.rs`, `security.rs`, `clipboard.rs`
- **SecureBuffer**: Custom Drop for complete memory zeroization
- **CommandResult enum**: Type-safe command execution flow
- **Raw mode terminal**: Full control over input/output

## ⚠️ Disclaimer

This tool is for **educational and ethical testing purposes only**. The authors are not responsible for misuse. Always obtain proper authorization before using security tools in any environment.

## 🛣️ Roadmap

### ✅ v0.1.0 - Initial Release

- [x] Process masking (Linux) as `systemd-journald`
- [x] Volatile command history (RAM only)
- [x] Ghost commands: `::status`, `::cp`, `::clear`, `::exit`, `::panic`
- [x] Secure memory handling with `zeroize`
- [x] Raw mode terminal with crossterm
- [x] Basic autocomplete (single match)
- [x] Command history navigation with arrow keys
- [x] Dynamic prompt with current directory

### ✅ v0.2.0 - Bug Fixes & History Management

- [x] Fix `::exit` bug with proper enum handling
- [x] Add `::history` command to view RAM-stored commands
- [x] Add `::purge-history` command with secure zeroization
- [x] Remove unused dependencies (reduced binary size)
- [x] CommandResult enum for type-safe execution flow
- [x] Comprehensive threat model documentation

### ✅ v0.3.0 - Advanced Security Features

- [x] Modularize code into separate files (`security.rs`, `clipboard.rs`)
- [x] Encrypted clipboard with ChaCha20Poly1305 (AEAD)
- [x] Auto-clear clipboard after 30 seconds
- [x] `::decrypt <key>` command to recover encrypted data
- [x] `::security-status` command with detailed analysis
- [x] Swap detection (warns if memory may be swapped to disk)
- [x] Monitoring tool detection (strace, gdb, auditd, eBPF, etc.)
- [x] ptrace detection (debugger attachment)

### ✅ v0.3.1 - Paranoid Mode

- [x] `::paranoid on|off` command for maximum security
- [x] Auto-panic when debugger is detected
- [x] Periodic security checks every 5 commands
- [x] Enhanced `::anti-debug` with auto-exit in paranoid mode
- [x] Command counter for security monitoring

### ✅ v0.3.2 - Security Audit Fixes

- [x] Custom Drop for complete history zeroization on exit
- [x] Base64 key zeroization after display
- [x] Comprehensive security audit (92% score)
- [x] SECURITY_AUDIT.md documentation

### ⏳ Planned - v0.4.0 (Short Term)

- [ ] Unit tests for security functions
- [ ] Memory locking (`mlock`) for sensitive buffers
- [ ] Core dump prevention (`madvise(MADV_DONTDUMP)`)
- [ ] Clipboard clear command (`::clear-clipboard`)
- [ ] Session key for persistent encryption

### 🔮 Future - v0.5.0+ (Long Term)

- [ ] Configuration file support (colors, prompt, timeout)
- [ ] Improved autocomplete (show multiple matches)
- [ ] Better UTF-8/grapheme cluster support
- [ ] Timing attack detection
- [ ] String obfuscation for sensitive constants
- [ ] Self-integrity checks (detect binary modification)
- [ ] Anti-VM/sandbox detection
- [ ] Network-based threat intelligence
- [ ] Plugin system for custom ghost commands

## 📄 License

This is a personal project for educational purposes. See LICENSE file for details.

---

<div align="center">

### 🔴 Built for Red Team Operations 🔴

[![Made with Rust](https://img.shields.io/badge/Made_with-Rust-b7410e?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Security First](https://img.shields.io/badge/Security-First-darkred?style=for-the-badge&logo=hackaday&logoColor=white)](.)
[![Zero Traces](https://img.shields.io/badge/Zero-Traces-black?style=for-the-badge&logo=ghost&logoColor=white)](.)

**👻 Stay Ghost. Stay Secure. 👻**

_For educational and authorized security research only._

</div>
