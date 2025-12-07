# Ghost Shell (gsh)

**Ghost Shell** is a secure, stealthy shell implementation in Rust designed for privacy and low-profile operations. It features process masking, secure memory handling, and built-in "ghost" commands for covert utilities.

## 🛡️ Key Features

*   **Process Masking (Linux):** Automatically disguises the process name as `systemd-journald` upon initialization to blend in with system processes.
*   **Secure Memory:** Utilizes the `zeroize` crate to ensure input buffers and sensitive data are scrubbed from memory when dropped.
*   **Ghost Commands (`::`):** A set of internal, prefixed commands that never touch the underlying system shell history.
*   **Clipboard Injection:** Securely copy text to the system clipboard directly from the shell without trace files.
*   **Raw Mode Interface:** Custom input handling using `crossterm` for a controlled terminal environment.

## 🚀 Installation

### Prerequisites

*   Rust and Cargo (latest stable version)
*   Linux environment (recommended for full feature support like process masking)
*   System dependencies for clipboard support (e.g., `libxcb`, `libx11` on Linux might be required by `arboard`)

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

### The Interface

Once running, you will see the `gsh>` prompt. You can type standard shell commands as usual:

```bash
gsh> ls -la
gsh> whoami
```

### 👻 Ghost Commands

Ghost commands are special instructions processed internally by the shell. They are prefixed with `::`.

| Command | Description |
| :--- | :--- |
| `::status` | displays the current security status of the shell. |
| `::cp <text>` | **Secure Copy:** Copies `<text>` directly to the system clipboard. |
| `::exit` | Terminates the Ghost Shell session. |
| `::panic` | *[Not Implemented]* "Nuclear" option placeholder. |

**Example:**
To copy a secret key to your clipboard without logging it to a history file or piping it through standard tools:

```bash
gsh> ::cp my-super-secret-password-123
```

## ⚠️ Disclaimer

This tool is for educational and ethical testing purposes only. The authors are not responsible for misuse.

## 📄 License

This is a personal project. Unauthorized copying or distribution of this software is strictly prohibited. All rights reserved.