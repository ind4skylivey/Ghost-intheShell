/// Encrypted clipboard module
/// Provides ephemeral, encrypted clipboard operations
use arboard::Clipboard;
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zeroize::Zeroize;

/// Encrypted clipboard manager
pub struct SecureClipboard {
    clipboard: Arc<Mutex<Clipboard>>,
    encryption_enabled: bool,
}

impl SecureClipboard {
    pub fn new(encryption_enabled: bool) -> Result<Self, String> {
        match Clipboard::new() {
            Ok(clipboard) => Ok(SecureClipboard {
                clipboard: Arc::new(Mutex::new(clipboard)),
                encryption_enabled,
            }),
            Err(e) => Err(format!("Failed to access clipboard: {}", e)),
        }
    }

    /// Copy text to clipboard with optional encryption and auto-clear
    pub fn copy_with_timeout(&self, mut text: String, timeout_secs: u64) -> Result<String, String> {
        let result = if self.encryption_enabled {
            self.copy_encrypted(&text, timeout_secs)
        } else {
            self.copy_plain(&text, timeout_secs)
        };

        // Zeroize the input text
        text.zeroize();
        result
    }

    /// Copy plain text with auto-clear
    fn copy_plain(&self, text: &str, timeout_secs: u64) -> Result<String, String> {
        let clipboard = Arc::clone(&self.clipboard);

        // Copy to clipboard
        {
            let mut cb = clipboard.lock().unwrap();
            cb.set_text(text)
                .map_err(|e| format!("Clipboard error: {}", e))?;
        }

        // Schedule auto-clear
        if timeout_secs > 0 {
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(timeout_secs));
                if let Ok(mut cb) = clipboard.lock() {
                    let _ = cb.clear();
                }
            });

            Ok(format!(
                "DATA INJECTED TO CLIPBOARD. AUTO-CLEAR IN {}s.",
                timeout_secs
            ))
        } else {
            Ok("DATA INJECTED TO CLIPBOARD. TRACES REMOVED.".to_string())
        }
    }

    /// Copy encrypted text with auto-clear
    fn copy_encrypted(&self, text: &str, timeout_secs: u64) -> Result<String, String> {
        // Generate random key and nonce
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);

        let cipher = ChaCha20Poly1305::new(&key_bytes.into());
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, text.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Encode as base64
        let encrypted_b64 = general_purpose::STANDARD.encode(ciphertext);
        let mut key_b64 = general_purpose::STANDARD.encode(key_bytes);
        let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);

        // Format: ENCRYPTED:<nonce>:<ciphertext>
        let clipboard_content = format!("GHOST_ENCRYPTED:{nonce_b64}:{encrypted_b64}");

        let clipboard = Arc::clone(&self.clipboard);

        // Copy to clipboard
        {
            let mut cb = clipboard.lock().unwrap();
            cb.set_text(&clipboard_content)
                .map_err(|e| format!("Clipboard error: {e}"))?;
        }

        // Schedule auto-clear
        if timeout_secs > 0 {
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(timeout_secs));
                if let Ok(mut cb) = clipboard.lock() {
                    let _ = cb.clear();
                }
            });
        }

        // Zeroize sensitive data
        key_bytes.zeroize();
        nonce_bytes.zeroize();

        // Create output message before zeroizing key_b64
        let output = format!(
            "ENCRYPTED DATA INJECTED. KEY: {key_b64}\r\nAUTO-CLEAR IN {timeout_secs}s.\r\nUse ::decrypt to recover."
        );

        // Zeroize the base64 key string
        key_b64.zeroize();

        Ok(output)
    }

    /// Decrypt clipboard content
    pub fn decrypt_clipboard(&self, key_b64: &str) -> Result<String, String> {
        let clipboard = Arc::clone(&self.clipboard);

        let clipboard_text = {
            let mut cb = clipboard.lock().unwrap();
            cb.get_text()
                .map_err(|e| format!("Failed to read clipboard: {}", e))?
        };

        if !clipboard_text.starts_with("GHOST_ENCRYPTED:") {
            return Err("Clipboard does not contain encrypted Ghost Shell data.".to_string());
        }

        let parts: Vec<&str> = clipboard_text
            .strip_prefix("GHOST_ENCRYPTED:")
            .unwrap()
            .split(':')
            .collect();

        if parts.len() != 2 {
            return Err("Invalid encrypted format.".to_string());
        }

        let nonce_b64 = parts[0];
        let ciphertext_b64 = parts[1];

        // Decode
        let mut key_bytes = general_purpose::STANDARD
            .decode(key_b64)
            .map_err(|_| "Invalid key format.")?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce_b64)
            .map_err(|_| "Invalid nonce format.")?;

        let ciphertext = general_purpose::STANDARD
            .decode(ciphertext_b64)
            .map_err(|_| "Invalid ciphertext format.")?;

        if key_bytes.len() != 32 || nonce_bytes.len() != 12 {
            key_bytes.zeroize();
            return Err("Invalid key or nonce length.".to_string());
        }

        // Decrypt
        let cipher = ChaCha20Poly1305::new(key_bytes.as_slice().into());
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
            key_bytes.zeroize();
            "Decryption failed. Wrong key or corrupted data.".to_string()
        })?;

        // Zeroize key
        key_bytes.zeroize();

        String::from_utf8(plaintext).map_err(|_| "Decrypted data is not valid UTF-8.".to_string())
    }

    /// Clear clipboard immediately
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), String> {
        let mut cb = self.clipboard.lock().unwrap();
        cb.clear()
            .map_err(|e| format!("Failed to clear clipboard: {}", e))
    }
}
