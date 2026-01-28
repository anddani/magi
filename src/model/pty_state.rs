//! State management for PTY-based command execution with credential handling.

use std::sync::mpsc::{Receiver, Sender};

use crate::git::credential::{CredentialResponse, CredentialType};
use crate::git::pty_command::PtyCommandResult;

/// State for an ongoing PTY command that may require credentials.
pub struct PtyState {
    /// Receiver for the command result (from PTY thread).
    pub result_rx: Receiver<PtyCommandResult>,
    /// Receiver for credential requests (from PTY thread).
    pub credential_request_rx: Receiver<CredentialType>,
    /// Sender for credential responses (to PTY thread).
    pub credential_response_tx: Sender<CredentialResponse>,
    /// Description of what operation is in progress (for UI feedback).
    pub operation: String,
}

impl PtyState {
    /// Creates a new PtyState for tracking an ongoing command.
    pub fn new(
        result_rx: Receiver<PtyCommandResult>,
        credential_request_rx: Receiver<CredentialType>,
        credential_response_tx: Sender<CredentialResponse>,
        operation: String,
    ) -> Self {
        Self {
            result_rx,
            credential_request_rx,
            credential_response_tx,
            operation,
        }
    }

    /// Checks for a credential request without blocking.
    /// Returns Some(CredentialType) if a credential is requested.
    pub fn check_credential_request(&self) -> Option<CredentialType> {
        self.credential_request_rx.try_recv().ok()
    }

    /// Sends a credential response to the PTY thread.
    /// Returns true if the response was sent successfully, false if the channel was closed.
    pub fn send_credential(&self, response: CredentialResponse) -> bool {
        self.credential_response_tx.send(response).is_ok()
    }

    /// Checks if the command has completed without blocking.
    /// Returns Some(result) if complete, None if still running.
    pub fn check_result(&self) -> Option<PtyCommandResult> {
        self.result_rx.try_recv().ok()
    }
}
