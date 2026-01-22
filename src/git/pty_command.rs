//! PTY-based command execution with credential prompt detection.
//!
//! This module provides functionality to run git commands in a pseudo-terminal (PTY),
//! allowing detection and handling of credential prompts that would otherwise hang
//! or fail when run with standard pipes.

use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use super::credential::{
    check_for_credential_request, CredentialResponse, CredentialStrategy, CredentialType,
};

/// Result of a PTY command execution.
#[derive(Debug)]
pub enum PtyCommandResult {
    /// Command completed successfully.
    Success {
        /// Combined stdout/stderr output.
        output: String,
    },
    /// Command failed with an error.
    Error {
        /// Error message or stderr output.
        message: String,
    },
    /// Command was terminated because credentials were required but strategy was Fail.
    CredentialRequired,
}

/// Channels for credential communication between PTY thread and UI.
pub struct CredentialChannels {
    /// Sender for credential requests (PTY thread -> UI).
    pub request_tx: Sender<CredentialType>,
    /// Receiver for credential responses (UI -> PTY thread).
    pub response_rx: Receiver<CredentialResponse>,
}

/// Channels held by the UI side for credential communication.
pub struct UiCredentialChannels {
    /// Receiver for credential requests (PTY thread -> UI).
    pub request_rx: Receiver<CredentialType>,
    /// Sender for credential responses (UI -> PTY thread).
    pub response_tx: Sender<CredentialResponse>,
}

/// Creates a pair of credential channel sets for PTY and UI communication.
pub fn create_credential_channels() -> (CredentialChannels, UiCredentialChannels) {
    let (request_tx, request_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();

    (
        CredentialChannels {
            request_tx,
            response_rx,
        },
        UiCredentialChannels {
            request_rx,
            response_tx,
        },
    )
}

/// Environment variables to force English locale for reliable pattern matching.
const LOCALE_ENV: [(&str, &str); 3] = [("LANG", "C"), ("LC_ALL", "C"), ("LC_MESSAGES", "C")];

/// Executes a git command in a PTY with credential handling.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository.
/// * `args` - Arguments to pass to git (e.g., `["push", "-v"]`).
/// * `strategy` - How to handle credential requests.
/// * `channels` - Channels for credential communication (only used if strategy is `Prompt`).
///
/// # Returns
///
/// The result of the command execution.
pub fn execute_git_with_pty<P: AsRef<Path>>(
    repo_path: P,
    args: &[&str],
    strategy: CredentialStrategy,
    channels: Option<CredentialChannels>,
) -> PtyCommandResult {
    // Create PTY system and pair
    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(pair) => pair,
        Err(e) => {
            return PtyCommandResult::Error {
                message: format!("Failed to open PTY: {}", e),
            };
        }
    };

    // Build command
    let mut cmd = CommandBuilder::new("git");
    cmd.arg("-C");
    cmd.arg(repo_path.as_ref());
    for arg in args {
        cmd.arg(*arg);
    }

    // Set locale environment for reliable pattern matching
    for (key, value) in LOCALE_ENV.iter() {
        cmd.env(*key, *value);
    }

    // Spawn the child process in the PTY
    let mut child = match pair.slave.spawn_command(cmd) {
        Ok(child) => child,
        Err(e) => {
            return PtyCommandResult::Error {
                message: format!("Failed to spawn command: {}", e),
            };
        }
    };

    // Drop the slave - we communicate through the master
    drop(pair.slave);

    // Get reader and writer from master
    let mut reader = match pair.master.try_clone_reader() {
        Ok(reader) => reader,
        Err(e) => {
            return PtyCommandResult::Error {
                message: format!("Failed to get PTY reader: {}", e),
            };
        }
    };

    let mut writer = match pair.master.take_writer() {
        Ok(writer) => writer,
        Err(e) => {
            return PtyCommandResult::Error {
                message: format!("Failed to get PTY writer: {}", e),
            };
        }
    };

    // Buffer for output and credential detection
    let mut output = String::new();
    let mut detection_buffer = String::new();
    let mut byte = [0u8; 1];

    // Read output byte by byte
    loop {
        match reader.read(&mut byte) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // Append to output (for display)
                if byte[0].is_ascii() {
                    output.push(byte[0] as char);
                }

                // Check for credential prompt
                if let Some(cred_type) = check_for_credential_request(&mut detection_buffer, byte[0])
                {
                    match strategy {
                        CredentialStrategy::Prompt => {
                            if let Some(ref channels) = channels {
                                // Send credential request to UI
                                if channels.request_tx.send(cred_type).is_err() {
                                    // UI channel closed, terminate
                                    let _ = child.kill();
                                    return PtyCommandResult::Error {
                                        message: "Credential channel closed".to_string(),
                                    };
                                }

                                // Wait for response from UI
                                match channels.response_rx.recv() {
                                    Ok(CredentialResponse::Input(input)) => {
                                        // Write credential to PTY (with newline)
                                        let input_with_newline = format!("{}\n", input);
                                        if writer.write_all(input_with_newline.as_bytes()).is_err() {
                                            return PtyCommandResult::Error {
                                                message: "Failed to write credential to PTY"
                                                    .to_string(),
                                            };
                                        }
                                        let _ = writer.flush();
                                    }
                                    Ok(CredentialResponse::Cancelled) => {
                                        // User cancelled - send empty line to fail auth
                                        let _ = writer.write_all(b"\n");
                                        let _ = writer.flush();
                                    }
                                    Err(_) => {
                                        // Channel closed, terminate
                                        let _ = child.kill();
                                        return PtyCommandResult::Error {
                                            message: "Credential response channel closed"
                                                .to_string(),
                                        };
                                    }
                                }
                            } else {
                                // No channels but Prompt strategy - shouldn't happen
                                let _ = child.kill();
                                return PtyCommandResult::Error {
                                    message: "Credential prompt detected but no channel provided"
                                        .to_string(),
                                };
                            }
                        }
                        CredentialStrategy::Fail => {
                            // Terminate the process
                            let _ = child.kill();
                            return PtyCommandResult::CredentialRequired;
                        }
                        CredentialStrategy::None => {
                            // Ignore - command may hang, but that's what was requested
                        }
                    }
                }
            }
            Err(e) => {
                // Check if it's a "would block" or similar non-fatal error
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                // Other errors might indicate the process ended
                break;
            }
        }
    }

    // Wait for the child to exit
    match child.wait() {
        Ok(status) => {
            if status.success() {
                PtyCommandResult::Success { output }
            } else {
                // Try to extract error message from output
                let error_msg = extract_error_message(&output);
                PtyCommandResult::Error {
                    message: error_msg.unwrap_or_else(|| "Command failed".to_string()),
                }
            }
        }
        Err(e) => PtyCommandResult::Error {
            message: format!("Failed to wait for command: {}", e),
        },
    }
}

/// Extracts an error message from git output.
/// Git typically writes errors to stderr, but with PTY they're mixed with stdout.
fn extract_error_message(output: &str) -> Option<String> {
    // Look for common git error patterns
    for line in output.lines().rev() {
        let line = line.trim();
        if line.starts_with("fatal:")
            || line.starts_with("error:")
            || line.starts_with("remote:")
            || line.contains("Permission denied")
            || line.contains("Authentication failed")
        {
            return Some(line.to_string());
        }
    }

    // If no specific error found, return last non-empty line
    output
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|s| s.trim().to_string())
}

/// Spawns a git command in a background thread with PTY and credential handling.
///
/// This is useful for non-blocking execution where the UI needs to remain responsive.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository.
/// * `args` - Arguments to pass to git.
/// * `strategy` - How to handle credential requests.
///
/// # Returns
///
/// A tuple of:
/// - `Receiver<PtyCommandResult>` - Receiver to get the command result.
/// - `Option<UiCredentialChannels>` - Channels for credential communication (if strategy is Prompt).
pub fn spawn_git_with_pty(
    repo_path: std::path::PathBuf,
    args: Vec<String>,
    strategy: CredentialStrategy,
) -> (Receiver<PtyCommandResult>, Option<UiCredentialChannels>) {
    let (result_tx, result_rx) = mpsc::channel();

    let (channels, ui_channels) = if strategy == CredentialStrategy::Prompt {
        let (c, u) = create_credential_channels();
        (Some(c), Some(u))
    } else {
        (None, None)
    };

    thread::spawn(move || {
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_git_with_pty(&repo_path, &args_refs, strategy, channels);
        let _ = result_tx.send(result);
    });

    (result_rx, ui_channels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_error_message_fatal() {
        let output = "Counting objects: 3\nfatal: could not read from remote repository";
        let msg = extract_error_message(output);
        assert_eq!(
            msg,
            Some("fatal: could not read from remote repository".to_string())
        );
    }

    #[test]
    fn test_extract_error_message_permission_denied() {
        let output = "Cloning...\nPermission denied (publickey).";
        let msg = extract_error_message(output);
        assert_eq!(msg, Some("Permission denied (publickey).".to_string()));
    }

    #[test]
    fn test_extract_error_message_auth_failed() {
        let output = "remote: Support for password authentication was removed.\nAuthentication failed for 'https://github.com/user/repo.git'";
        let msg = extract_error_message(output);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("Authentication failed"));
    }

    #[test]
    fn test_extract_error_message_fallback() {
        let output = "Something went wrong\n";
        let msg = extract_error_message(output);
        assert_eq!(msg, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_extract_error_message_empty() {
        let output = "";
        let msg = extract_error_message(output);
        assert!(msg.is_none());
    }
}
