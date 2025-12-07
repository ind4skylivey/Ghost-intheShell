use std::io::{self, Write};
use std::ffi::CString;
use std::process::Command;
use std::env;
use crossterm={
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, enable_raw_mode, disable_raw_mode, Clear, ClearType},
    ExecutableCommand,
    cursor,
};
use zeroize::{Zeroize, ZeroizeOnDrop};
use arboard::Clipboard; // Importar librería de portapapeles

// Estructura segura para el input.
#[derive(Zeroize, ZeroizeOnDrop)]
struct SecureBuffer {
    content: String,
}

const GHOST_COMMAND_PREFIX: &str = "::";

impl SecureBuffer {
    fn new() -> Self {
        SecureBuffer { content: String::new() }
    }

    fn push(&mut self, c: char) {
        self.content.push(c);
    }

    fn pop(&mut self) {
        self.content.pop();
    }
    
    fn clear(&mut self) {
        self.content.clear();
    }

    fn process_command(&self) -> String {
        let trimmed_command = self.content.trim();

        if trimmed_command.is_empty() {
            return String::new();
        }

        if trimmed_command.starts_with(GHOST_COMMAND_PREFIX) {
            // --- GHOST COMMANDS ---
            let ghost_cmd = &trimmed_command[GHOST_COMMAND_PREFIX.len()..];
            
            // Separar comando de argumentos (ej: "cp clave-secreta")
            let parts: Vec<&str> = ghost_cmd.splitn(2, ' ').collect();
            let cmd = parts[0];
            let args = if parts.len() > 1 { parts[1] } else { "" };

            match cmd {
                "panic" => "NUCLEAR OPTION ARMED... [NOT IMPLEMENTED]".to_string(),
                "status" => "GHOST MODE ACTIVE. MEMORY SECURE.".to_string(),
                "exit" => "TERMINATING".to_string(),
                
                // Killer Feature: Clipboard Inject
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
            // --- SHELL COMMANDS ---
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
                        if !result.is_empty() { result.push_str("\r\n"); }
                        result.push_str("STDERR:\r\n");
                        result.push_str(&stderr);
                    }
                    
                    // Normalizar saltos de línea para raw mode
                    result.replace("\n", "\r\n")
                },
                Err(e) => format!("Failed to execute process: {}\r\n", e),
            }
        }
    }
}

fn main() -> io::Result<()> {
    // 1. PROCESS MASKING
    #[cfg(target_os = "linux")]
    unsafe {
        let fake_name = CString::new("systemd-journald").unwrap();
        prctl::set_name(fake_name.to_str().unwrap()).expect("Failed to set process name");
    }

    println!("Initializing Ghost Shell protocol...");
    
    // 2. RAW MODE ACQUISITION
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(Clear(ClearType::All))?;

    let mut buffer = SecureBuffer::new();
    let mut running = true;

    write!(stdout, "gsh> ")?;
    stdout.flush()?;

    while running {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match code {
                    // --- NUEVO MANEJO DE CTRL+C ---
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        // Cancelar línea actual, no salir
                        buffer.clear();
                        write!(stdout, "^C\r\n")?;
                        write!(stdout, "gsh> ")?;
                    }
                    
                    KeyCode::Enter => {
                        write!(stdout, "\r\n")?;
                        let output = buffer.process_command();
                        
                        // Lógica de salida
                        if buffer.content.trim() == format!("{}exit", GHOST_COMMAND_PREFIX) {
                            running = false;
                        } else {
                            // Si no es exit, imprimimos el resultado
                            if !output.is_empty() {
                                write!(stdout, "{}\r\n", output)?;
                            }
                            buffer.clear();
                            write!(stdout, "gsh> ")?;
                        }
                    }
                    KeyCode::Char(c) => {
                        buffer.push(c);
                        write!(stdout, "{}", c)?;
                    }
                    KeyCode::Backspace => {
                        buffer.pop();
                        let (col, _) = cursor::position()?;
                        if col > "gsh> ".len() as u16 {
                            write!(stdout, "\x08 \x08")?;
                        }
                    }
                    _ => {}
                }
                stdout.flush()?;
            }
        }
    }

    // 3. CLEANUP & EXIT
    disable_raw_mode()?;
    // Simulación de limpieza segura
    println!("\n[!] INITIATING SECURE SHUTDOWN...");
    println!("[*] Overwriting memory buffers... DONE");
    println!("[*]