# Ghost Shell (gsh)

**Ghost Shell** is a secure, stealthy shell implementation in Rust designed for privacy and low-profile operations. It features process masking, secure memory handling, and built-in "ghost" commands for covert utilities.

## 🛡️ Key Features

*   **Process Masking (Linux):** Automatically disguises the process name as `systemd-journald` upon initialization to blend in with system processes.
*   **Secure Memory:** Utilizes the `zeroize` crate to ensure input buffers and sensitive data are scrubbed from memory when dropped.
*   **Volatile History:** Command history is kept strictly in RAM and is never written to disk (`.bash_history` etc.), ensuring no forensic trace remains after exit.
*   **Ghost Commands (`::`):** A set of internal, prefixed commands that never touch the underlying system shell history.
*   **Clipboard Injection:** Securely copy text to the system clipboard directly from the shell without trace files.
*   **Dynamic Prompt:** Displays your current directory context `gsh <dir>>` while keeping a low profile.

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

### Navigation & UX

*   **CD:** Native support for `cd` to change directories (e.g., `cd /tmp`, `cd ..`, `cd ~`).
*   **Cursor:** Use `←` / `→` arrows to edit your command line.
*   **History:** Use `↑` / `↓` arrows to cycle through previous commands (RAM only).
*   **Autocomplete:** Press `Tab` to auto-complete filenames in the current directory.
*   **Clear:** `Ctrl+L` or `clear` to clean the screen.

### 👻 Ghost Commands

Ghost commands are special instructions processed internally by the shell. They are prefixed with `::`.

| Command | Description |
| :--- | :--- |
| `::status` | Displays the current security status of the shell. |
| `::cp <text>` | **Secure Copy:** Copies `<text>` directly to the system clipboard. |
| `::exit` | Terminates the Ghost Shell session. |
| `::clear` | Clears the terminal screen securely. |
| `::panic` | **NUCLEAR OPTION:** Simulates a crash, wipes memory, and exits immediately. |

**Example:**
To copy a secret key to your clipboard without logging it to a history file or piping it through standard tools:

```bash
gsh ~/secrets> ::cp my-super-secret-password-123
```

## ⚠️ Disclaimer

This tool is for educational and ethical testing purposes only. The authors are not responsible for misuse.

## 📄 License

This is a personal project. Unauthorized copying or distribution of this software is strictly prohibited. All rights reserved.
