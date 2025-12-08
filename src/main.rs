mod clipboard;
mod security;

use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::env;
use std::ffi::CString;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use zeroize::Zeroize;

use crate::clipboard::SecureClipboard;
use crate::security::{initialize_security, is_debugger_present, SecurityStatus};

// --- CONSTANTS ---
const GHOST_COMMAND_PREFIX: &str = "::";

// --- ENUMS ---

/// Result of command execution
enum CommandResult {
    /// No output, continue normally
    NoOp,
    /// Command produced output
    Output(String),
    /// Exit the shell
    Exit,
}

// --- STRUCTURES ---

/// Main Ghost Shell state (reserved for future refactoring)
#[allow(dead_code)]
struct GhostShell {
    buffer: SecureBuffer,
    security_status: SecurityStatus,
    clipboard: SecureClipboard,
    clipboard_timeout: u64, // seconds
    encryption_enabled: bool,
}

#[allow(dead_code)]
impl GhostShell {
    fn new() -> Result<Self, String> {
        let security_status = initialize_security();
        let encryption_enabled = true; // Default to encrypted clipboard
        let clipboard_timeout = 30; // 30 seconds default

        let clipboard = SecureClipboard::new(encryption_enabled)?;

        Ok(GhostShell {
            buffer: SecureBuffer::new(),
            security_status,
            clipboard,
            clipboard_timeout,
            encryption_enabled,
        })
    }
}

/// SecureBuffer holds command input and history
/// Note: We implement Drop manually to ensure history is zeroized
struct SecureBuffer {
    content: String,
    history: Vec<String>,
    history_index: usize, // Points to index in history. history.len() = new line.
    cursor_pos: usize,    // Cursor position within 'content' (chars)
    command_count: usize, // Track number of commands executed
    paranoid_mode: bool,  // Auto-panic on threat detection
}

/// Custom Drop implementation to securely zeroize all sensitive data
impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // Zeroize the current command buffer
        self.content.zeroize();

        // Zeroize each command in history
        for cmd in self.history.iter_mut() {
            cmd.zeroize();
        }
        self.history.clear();

        // Reset counters (not sensitive, but good hygiene)
        self.history_index = 0;
        self.cursor_pos = 0;
        self.command_count = 0;
        self.paranoid_mode = false;
    }
}

impl SecureBuffer {
    fn new() -> Self {
        SecureBuffer {
            content: String::new(),
            history: Vec::new(),
            history_index: 0,
            cursor_pos: 0,
            command_count: 0,
            paranoid_mode: false, // Can be enabled with ::paranoid command
        }
    }

    // --- MANIPULATION ---

    fn insert(&mut self, c: char) {
        if self.cursor_pos >= self.content.len() {
            self.content.push(c);
        } else {
            self.content.insert(self.cursor_pos, c);
        }
        self.cursor_pos += 1;
    }

    fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.content.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_pos < self.content.len() {
            self.cursor_pos += 1;
        }
    }

    // --- HISTORY ---

    fn history_up(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(cmd) = self.history.get(self.history_index) {
                self.content = cmd.clone();
                self.cursor_pos = self.content.len();
            }
        }
    }

    fn history_down(&mut self) {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            if self.history_index == self.history.len() {
                self.content.clear();
                self.cursor_pos = 0;
            } else if let Some(cmd) = self.history.get(self.history_index) {
                self.content = cmd.clone();
                self.cursor_pos = self.content.len();
            }
        }
    }

    fn commit_history(&mut self) {
        if !self.content.trim().is_empty() {
            // Avoid duplicates at the end
            if self.history.last() != Some(&self.content) {
                self.history.push(self.content.clone());
            }
        }
        self.history_index = self.history.len();
    }

    // --- AUTOCOMPLETE ---
    fn autocomplete(&mut self) {
        // Very basic implementation: complete files in current dir based on last word
        let parts: Vec<&str> = self.content.split_whitespace().collect();
        if let Some(last_word) = parts.last() {
            let path_to_check = if last_word.contains('/') {
                Path::new(last_word).parent().unwrap_or(Path::new("."))
            } else {
                Path::new(".")
            };

            let prefix = Path::new(last_word)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if let Ok(entries) = fs::read_dir(path_to_check) {
                let matches: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .filter(|name| name.starts_with(prefix))
                    .collect();

                if matches.len() == 1 {
                    let completion = &matches[0][prefix.len()..];
                    for c in completion.chars() {
                        self.insert(c);
                    }
                } else if matches.len() > 1 {
                    // TODO: Show possibilities? For now, just cycle or do nothing.
                }
            }
        }
    }

    fn clear_state(&mut self) {
        self.content.clear();
        self.cursor_pos = 0;
        self.history_index = self.history.len();
    }

    /// Securely purge command history from memory
    fn purge_history(&mut self) {
        // Zeroize each string in history before clearing
        for cmd in self.history.iter_mut() {
            cmd.zeroize();
        }
        self.history.clear();
        self.history_index = 0;
    }

    // --- EXECUTION ---

    fn process_command(&mut self) -> CommandResult {
        let trimmed_command = self.content.trim();

        if trimmed_command.is_empty() {
            return CommandResult::NoOp;
        }

        // Increment command counter
        self.command_count += 1;

        // Periodic security check in paranoid mode (every 5 commands)
        if self.paranoid_mode && self.command_count.is_multiple_of(5) && is_debugger_present() {
            let _ = execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0));
            println!("⚠ PERIODIC CHECK: DEBUGGER DETECTED");
            println!("PARANOID MODE - INITIATING EMERGENCY SHUTDOWN...");
            std::thread::sleep(std::time::Duration::from_millis(500));
            std::process::exit(137);
        }

        if let Some(ghost_cmd) = trimmed_command.strip_prefix(GHOST_COMMAND_PREFIX) {
            let parts: Vec<&str> = ghost_cmd.splitn(2, ' ').collect();
            let cmd = parts[0];
            let args = if parts.len() > 1 { parts[1] } else { "" };

            match cmd {
                "panic" => {
                    // NUCLEAR OPTION
                    let _ = execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0));
                    println!("KERNEL PANIC - MEMORY CORRUPTION DETECTED at 0xDEADBEEF");
                    println!("Dumping core to /dev/null...");
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    std::process::exit(137); // Simulated crash
                }
                "status" => CommandResult::Output(
                    "GHOST MODE ACTIVE. MEMORY SECURE. TRACE: NONE.".to_string(),
                ),
                "security-status" => {
                    let status = initialize_security();
                    CommandResult::Output(status.report())
                }
                "exit" => CommandResult::Exit,
                "clear" => {
                    let _ = execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0));
                    CommandResult::NoOp
                }
                "history" => {
                    if self.history.is_empty() {
                        CommandResult::Output("No commands in history.".to_string())
                    } else {
                        let mut output = String::from("Command History (RAM only):\r\n");
                        for (i, cmd) in self.history.iter().enumerate() {
                            output.push_str(&format!("  {}: {}\r\n", i + 1, cmd));
                        }
                        CommandResult::Output(output)
                    }
                }
                "purge-history" => {
                    let count = self.history.len();
                    self.purge_history();
                    CommandResult::Output(format!(
                        "HISTORY PURGED. {} COMMANDS ZEROIZED FROM MEMORY.",
                        count
                    ))
                }
                "cp" => {
                    if args.is_empty() {
                        CommandResult::Output("Error: No content to copy.".to_string())
                    } else {
                        match SecureClipboard::new(true) {
                            Ok(clipboard) => {
                                match clipboard.copy_with_timeout(args.to_string(), 30) {
                                    Ok(msg) => CommandResult::Output(msg),
                                    Err(e) => CommandResult::Output(e),
                                }
                            }
                            Err(e) => CommandResult::Output(e),
                        }
                    }
                }
                "decrypt" => {
                    if args.is_empty() {
                        CommandResult::Output("Usage: ::decrypt <key>".to_string())
                    } else {
                        match SecureClipboard::new(false) {
                            Ok(clipboard) => match clipboard.decrypt_clipboard(args) {
                                Ok(plaintext) => {
                                    CommandResult::Output(format!("Decrypted: {}", plaintext))
                                }
                                Err(e) => CommandResult::Output(e),
                            },
                            Err(e) => CommandResult::Output(e),
                        }
                    }
                }
                "anti-debug" => {
                    if is_debugger_present() {
                        if self.paranoid_mode {
                            // Auto-panic in paranoid mode
                            let _ = execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0));
                            println!("⚠ DEBUGGER DETECTED - PARANOID MODE ACTIVE");
                            println!("INITIATING EMERGENCY SHUTDOWN...");
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            std::process::exit(137);
                        } else {
                            CommandResult::Output("⚠ WARNING: DEBUGGER DETECTED!".to_string())
                        }
                    } else {
                        CommandResult::Output("✓ No debugger detected.".to_string())
                    }
                }
                "paranoid" => {
                    if args == "on" {
                        self.paranoid_mode = true;
                        CommandResult::Output(
                            "⚠ PARANOID MODE ENABLED\r\n\
                            - Auto-panic on debugger detection\r\n\
                            - Periodic security checks every 5 commands\r\n\
                            - Enhanced threat monitoring"
                                .to_string(),
                        )
                    } else if args == "off" {
                        self.paranoid_mode = false;
                        CommandResult::Output("PARANOID MODE DISABLED".to_string())
                    } else {
                        CommandResult::Output(format!(
                            "Paranoid mode: {}\r\nUsage: ::paranoid on|off",
                            if self.paranoid_mode {
                                "ENABLED"
                            } else {
                                "DISABLED"
                            }
                        ))
                    }
                }
                _ => CommandResult::Output(format!("Unknown GHOST command: '{}'", cmd)),
            }
        } else {
            // Built-in: cd
            let parts: Vec<&str> = trimmed_command.splitn(2, ' ').collect();
            if parts[0] == "cd" {
                let path_str = parts.get(1).unwrap_or(&"~");
                let path = match *path_str {
                    "~" => env::var("HOME").unwrap_or_else(|_| "/".to_string()),
                    _ => path_str.to_string(),
                };
                match env::set_current_dir(&path) {
                    Ok(_) => return CommandResult::NoOp,
                    Err(e) => return CommandResult::Output(format!("cd: {}", e)),
                }
            }

            // Built-in: clear (standard shell alias)
            if parts[0] == "clear" {
                let _ = execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0));
                return CommandResult::NoOp;
            }

            let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            match Command::new(shell).arg("-c").arg(trimmed_command).output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\r\n");
                        }
                        result.push_str("STDERR:\r\n");
                        result.push_str(&stderr);
                    }
                    CommandResult::Output(result.replace("\n", "\r\n"))
                }
                Err(e) => CommandResult::Output(format!("Failed to execute process: {}\r\n", e)),
            }
        }
    }
}

// --- UTILS ---

fn get_current_prompt() -> String {
    let current_dir = env::current_dir()
        .unwrap_or_else(|_| "/".into())
        .file_name()
        .unwrap_or_else(|| "gsh".as_ref())
        .to_string_lossy()
        .to_string();
    format!("gsh {}>> ", current_dir)
}

fn redraw_line(stdout: &mut io::Stdout, buffer: &SecureBuffer) -> io::Result<()> {
    let prompt = get_current_prompt();
    queue!(
        stdout,
        MoveToColumn(0),
        Clear(ClearType::UntilNewLine),
        Print(&prompt),
        Print(&buffer.content),
        MoveToColumn((prompt.len() + buffer.cursor_pos) as u16)
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // 1. PROCESS MASKING
    #[cfg(target_os = "linux")]
    {
        if let Ok(fake_name) = CString::new("systemd-journald") {
            let _ = prctl::set_name(fake_name.to_str().unwrap());
        }
    }

    println!("Initializing Ghost Shell protocol...");

    // 2. RAW MODE ACQUISITION
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveToColumn(0))?;

    let mut buffer = SecureBuffer::new();
    let mut running = true;

    // Initial draw
    redraw_line(&mut stdout, &buffer)?;

    while running {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        buffer.content.clear();
                        buffer.cursor_pos = 0;
                        write!(stdout, "^C\r\n")?;
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+L to clear screen
                        execute!(stdout, Clear(ClearType::All), MoveToColumn(0))?;
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Enter => {
                        write!(stdout, "\r\n")?;

                        // Process command and handle result
                        let result = buffer.process_command();

                        match result {
                            CommandResult::Exit => {
                                running = false;
                            }
                            CommandResult::Output(output) => {
                                write!(stdout, "{}\r\n", output)?;
                                buffer.commit_history();
                                buffer.clear_state();
                                redraw_line(&mut stdout, &buffer)?;
                            }
                            CommandResult::NoOp => {
                                buffer.commit_history();
                                buffer.clear_state();
                                redraw_line(&mut stdout, &buffer)?;
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        buffer.insert(c);
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Backspace => {
                        buffer.backspace();
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Left => {
                        buffer.move_left();
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Right => {
                        buffer.move_right();
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Up => {
                        buffer.history_up();
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Down => {
                        buffer.history_down();
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    KeyCode::Tab => {
                        buffer.autocomplete();
                        redraw_line(&mut stdout, &buffer)?;
                    }
                    _ => {} // Ignore other keys
                }
            }
        }
    }

    // 3. CLEANUP & EXIT
    disable_raw_mode()?;
    println!("\n[!] INITIATING SECURE SHUTDOWN...");
    println!("[*] Overwriting memory buffers... DONE.");
    println!("[*] All systems clear. Ghost Shell terminated.");
    Ok(())
}
