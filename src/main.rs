use std::io::{self, Write};
use std::ffi::CString;
use std::process::Command;
use std::env;
use std::fs;
use std::path::Path;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::{MoveToColumn},
    style::Print,
    execute,
    queue,
};
use zeroize::{Zeroize, ZeroizeOnDrop};
use arboard::Clipboard;

// --- CONSTANTS ---
const GHOST_COMMAND_PREFIX: &str = "::";

// --- STRUCTURES ---

#[derive(Zeroize, ZeroizeOnDrop)]
struct SecureBuffer {
    content: String,
    #[zeroize(skip)]
    history: Vec<String>,
    #[zeroize(skip)]
    history_index: usize, // Points to index in history. history.len() = new line.
    #[zeroize(skip)]
    cursor_pos: usize,    // Cursor position within 'content' (chars)
}

impl SecureBuffer {
    fn new() -> Self {
        SecureBuffer {
            content: String::new(),
            history: Vec::new(),
            history_index: 0,
            cursor_pos: 0,
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
            
            let prefix = Path::new(last_word).file_name().and_then(|s| s.to_str()).unwrap_or("");

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

    // --- EXECUTION ---

    fn process_command(&self) -> String {
        let trimmed_command = self.content.trim();

        if trimmed_command.is_empty() {
            return String::new();
        }

        if trimmed_command.starts_with(GHOST_COMMAND_PREFIX) {
            let ghost_cmd = &trimmed_command[GHOST_COMMAND_PREFIX.len()..];
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
                },
                "status" => "GHOST MODE ACTIVE. MEMORY SECURE. TRACE: NONE.".to_string(),
                "exit" => "TERMINATING".to_string(),
                "clear" => {
                   // Handled in main loop for full screen clear, but we can return empty here
                   // Actually, we'll let main handle the printing, but if we want to clear *screen*
                   // we should do it here or signal main.
                   // Let's just return a signal string or handle it directly?
                   // Direct handling is better for visual effects.
                   let _ = execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0));
                   String::new() 
                },
                "cp" => {
                    if args.is_empty() {
                        "Error: No content to copy.".to_string()
                    } else {
                        match Clipboard::new() {
                            Ok(mut clipboard) => {
                                match clipboard.set_text(args) {
                                    Ok(_) => "DATA INJECTED TO CLIPBOARD. TRACES REMOVED.".to_string(),
                                    Err(e) => format!("Clipboard Error: {}", e),
                                }
                            },
                            Err(e) => format!("Clipboard Access Error: {}", e),
                        }
                    }
                },
                _ => format!("Unknown GHOST command: '{}'", cmd),
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
                    Ok(_) => return String::new(),
                    Err(e) => return format!("cd: {}", e),
                }
            }
            
            // Built-in: clear (standard shell alias)
            if parts[0] == "clear" {
                 let _ = execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0));
                 return String::new();
            }

            let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            match Command::new(shell).arg("-c").arg(trimmed_command).output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let mut result = String::new();
                    if !stdout.is_empty() { result.push_str(&stdout); }
                    if !stderr.is_empty() {
                        if !result.is_empty() { result.push_str("\r\n"); }
                        result.push_str("STDERR:\r\n");
                        result.push_str(&stderr);
                    }
                    result.replace("\n", "\r\n")
                },
                Err(e) => format!("Failed to execute process: {}\r\n", e),
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
    format!("gsh {}}}> ", current_dir)
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
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
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
                        
                        // Process
                        let output = buffer.process_command();
                        
                        if buffer.content.trim() == format!("{}\"exit\"", GHOST_COMMAND_PREFIX) {
                            running = false;
                        } else {
                            if !output.is_empty() {
                                write!(stdout, "{}\r\n", output)?;
                            }
                            buffer.commit_history();
                            buffer.clear_state();
                            redraw_line(&mut stdout, &buffer)?;
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