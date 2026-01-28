//! Credential detection types and patterns for PTY-based command execution.
//!
//! This module provides the types and regex patterns needed to detect credential
//! prompts from commands like `git push`, `git fetch`, and SSH operations.

use lazy_static::lazy_static;
use regex::Regex;

const BUFFER_LEN: usize = 256;

/// Types of credentials that can be requested by commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialType {
    /// HTTPS username - NOT masked in UI
    Username,
    /// HTTPS password - masked in UI
    Password,
    /// SSH key passphrase - masked in UI
    Passphrase,
    /// Hardware security key PIN (PKCS11/PIV) - masked in UI
    Pin,
    /// 2FA token - masked in UI
    Token,
}

impl CredentialType {
    /// Returns the display title for the credential prompt popup.
    pub fn display_title(&self) -> &'static str {
        match self {
            CredentialType::Username => "Username",
            CredentialType::Password => "Password",
            CredentialType::Passphrase => "Passphrase",
            CredentialType::Pin => "PIN",
            CredentialType::Token => "2FA Token",
        }
    }

    /// Returns whether this credential type should be masked in the UI.
    pub fn should_mask(&self) -> bool {
        match self {
            CredentialType::Username => false,
            CredentialType::Password => true,
            CredentialType::Passphrase => true,
            CredentialType::Pin => true,
            CredentialType::Token => true,
        }
    }
}

/// Strategy for handling credential requests during command execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CredentialStrategy {
    /// No credential handling; command may hang if prompted.
    #[default]
    None,
    /// Show UI popup when credentials requested.
    Prompt,
    /// Terminate process gracefully if credentials requested.
    Fail,
}

/// Response from the UI after a credential request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialResponse {
    /// User entered a value (may be empty).
    Input(String),
    /// User cancelled the prompt.
    Cancelled,
}

lazy_static! {
    /// Compiled regex patterns for detecting credential prompts.
    /// Each pattern is paired with its corresponding credential type.
    pub static ref CREDENTIAL_PATTERNS: Vec<(Regex, CredentialType)> = vec![
        // Git credential helper - username
        (Regex::new(r"Username\s*for\s*'.+':").unwrap(), CredentialType::Username),
        // Git credential helper - password
        (Regex::new(r"Password\s*for\s*'.+':").unwrap(), CredentialType::Password),
        // SSH password auth
        (Regex::new(r".+'s password:").unwrap(), CredentialType::Password),
        // Generic password prompt
        (Regex::new(r"Password:").unwrap(), CredentialType::Password),
        // SSH key passphrase
        (Regex::new(r"Enter\s*passphrase\s*for\s*key\s*'.+':").unwrap(), CredentialType::Passphrase),
        // PKCS11/security key PIN
        (Regex::new(r"Enter\s*PIN\s*for\s*.+\s*key\s*.+:").unwrap(), CredentialType::Pin),
        // PIV/smart card PIN
        (Regex::new(r"Enter\s*PIN\s*for\s*'.+':").unwrap(), CredentialType::Pin),
        // 2FA token prompts
        (Regex::new(r".*2FA Token.*").unwrap(), CredentialType::Token),
    ];
}

/// Checks if the buffer contains a credential prompt.
///
/// This function is called byte-by-byte as output is read from the PTY.
/// When a newline is encountered, content before it is discarded since
/// prompts don't span lines.
///
/// Returns the detected credential type if a pattern matches.
pub fn check_for_credential_request(buffer: &mut String, new_byte: u8) -> Option<CredentialType> {
    // Only process printable ASCII and common control characters
    if new_byte.is_ascii() {
        buffer.push(new_byte as char);
    }

    // Check all patterns against the current buffer
    for (pattern, cred_type) in CREDENTIAL_PATTERNS.iter() {
        if pattern.is_match(buffer) {
            buffer.clear();
            return Some(*cred_type);
        }
    }

    // On newline, keep only content after the newline
    // (credential prompts don't span multiple lines)
    if let Some(idx) = buffer.rfind('\n') {
        *buffer = buffer[idx + 1..].to_string();
    }

    // Prevent buffer from growing unbounded (keep last `BUFFER_LEN` bytes)
    if buffer.len() > BUFFER_LEN {
        let start = buffer.len() - BUFFER_LEN;
        *buffer = buffer[start..].to_string();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_type_display_title() {
        assert_eq!(CredentialType::Username.display_title(), "Username");
        assert_eq!(CredentialType::Password.display_title(), "Password");
        assert_eq!(CredentialType::Passphrase.display_title(), "Passphrase");
        assert_eq!(CredentialType::Pin.display_title(), "PIN");
        assert_eq!(CredentialType::Token.display_title(), "2FA Token");
    }

    #[test]
    fn test_credential_type_should_mask() {
        assert!(!CredentialType::Username.should_mask());
        assert!(CredentialType::Password.should_mask());
        assert!(CredentialType::Passphrase.should_mask());
        assert!(CredentialType::Pin.should_mask());
        assert!(CredentialType::Token.should_mask());
    }

    #[test]
    fn test_detect_git_username_prompt() {
        let mut buffer = String::new();
        let prompt = "Username for 'https://github.com':";

        for byte in prompt.bytes() {
            if let Some(cred_type) = check_for_credential_request(&mut buffer, byte) {
                assert_eq!(cred_type, CredentialType::Username);
                return;
            }
        }
        panic!("Expected to detect username prompt");
    }

    #[test]
    fn test_detect_git_password_prompt() {
        let mut buffer = String::new();
        let prompt = "Password for 'https://user@github.com':";

        for byte in prompt.bytes() {
            if let Some(cred_type) = check_for_credential_request(&mut buffer, byte) {
                assert_eq!(cred_type, CredentialType::Password);
                return;
            }
        }
        panic!("Expected to detect password prompt");
    }

    #[test]
    fn test_detect_ssh_password_prompt() {
        let mut buffer = String::new();
        let prompt = "user@host's password:";

        for byte in prompt.bytes() {
            if let Some(cred_type) = check_for_credential_request(&mut buffer, byte) {
                assert_eq!(cred_type, CredentialType::Password);
                return;
            }
        }
        panic!("Expected to detect SSH password prompt");
    }

    #[test]
    fn test_detect_ssh_passphrase_prompt() {
        let mut buffer = String::new();
        let prompt = "Enter passphrase for key '/home/user/.ssh/id_rsa':";

        for byte in prompt.bytes() {
            if let Some(cred_type) = check_for_credential_request(&mut buffer, byte) {
                assert_eq!(cred_type, CredentialType::Passphrase);
                return;
            }
        }
        panic!("Expected to detect passphrase prompt");
    }

    #[test]
    fn test_detect_generic_password_prompt() {
        let mut buffer = String::new();
        let prompt = "Password:";

        for byte in prompt.bytes() {
            if let Some(cred_type) = check_for_credential_request(&mut buffer, byte) {
                assert_eq!(cred_type, CredentialType::Password);
                return;
            }
        }
        panic!("Expected to detect generic password prompt");
    }

    #[test]
    fn test_newline_clears_buffer() {
        let mut buffer = String::new();

        // Add some non-matching text
        for byte in "Some output text\n".bytes() {
            check_for_credential_request(&mut buffer, byte);
        }

        // Buffer should be cleared after newline
        assert!(buffer.is_empty() || !buffer.contains("Some output"));

        // Now add a password prompt - should still be detected
        let prompt = "Password:";
        for byte in prompt.bytes() {
            if let Some(cred_type) = check_for_credential_request(&mut buffer, byte) {
                assert_eq!(cred_type, CredentialType::Password);
                return;
            }
        }
        panic!("Expected to detect password prompt after newline");
    }

    #[test]
    fn test_no_false_positive_on_normal_output() {
        let mut buffer = String::new();
        let output = "Counting objects: 100% (10/10), done.\n\
                      Delta compression using up to 8 threads.\n\
                      Compressing objects: 100% (5/5), done.\n\
                      Writing objects: 100% (5/5), 500 bytes | 500.00 KiB/s, done.\n";

        for byte in output.bytes() {
            let result = check_for_credential_request(&mut buffer, byte);
            assert!(result.is_none(), "False positive detected");
        }
    }

    #[test]
    fn test_buffer_size_limit() {
        let mut buffer = String::new();

        // Add more than 256 bytes without a match
        let long_string = "x".repeat(BUFFER_LEN + 10);
        for byte in long_string.bytes() {
            check_for_credential_request(&mut buffer, byte);
        }

        // Buffer should be truncated to prevent unbounded growth
        assert!(buffer.len() <= BUFFER_LEN);
    }
}
