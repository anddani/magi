use std::path::Path;
use std::process::Command;

use super::git_cmd;

/// Returns the gpg program git is configured to use (`gpg.program`),
/// falling back to "gpg".
fn gpg_program<P: AsRef<Path>>(repo_path: P) -> String {
    match git_cmd(&repo_path, &["config", "--get", "gpg.program"]).output() {
        Ok(out) if out.status.success() => {
            let program = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if program.is_empty() {
                "gpg".to_string()
            } else {
                program
            }
        }
        _ => "gpg".to_string(),
    }
}

/// Lists the secret gpg keys available for signing as "<keyid> <user id>"
/// entries, mirroring magit's `magit-read-gpg-signing-key`. Returns an
/// empty list when gpg is unavailable, so the caller can fall back to
/// free-text input.
pub fn list_secret_keys<P: AsRef<Path>>(repo_path: P) -> Vec<String> {
    let output = Command::new(gpg_program(repo_path))
        .args(["--list-secret-keys", "--with-colons"])
        .output();
    match output {
        Ok(out) if out.status.success() => parse_secret_keys(&String::from_utf8_lossy(&out.stdout)),
        _ => vec![],
    }
}

/// Parses `gpg --list-secret-keys --with-colons` output into
/// "<keyid> <user id>" entries, one per `sec` record. The keyid is field 5
/// of the `sec` line and the user id is field 10 of the following `uid`
/// line (or of the `sec` line itself in older gpg versions).
fn parse_secret_keys(output: &str) -> Vec<String> {
    let mut keys = Vec::new();
    // Keyid of the current `sec` record, until its uid is found
    let mut pending: Option<String> = None;
    for line in output.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        match fields.first() {
            Some(&"sec") => {
                if let Some(keyid) = pending.take() {
                    keys.push(keyid);
                }
                let keyid = fields.get(4).copied().unwrap_or("");
                if keyid.is_empty() {
                    continue;
                }
                match fields.get(9).copied().filter(|uid| !uid.is_empty()) {
                    Some(uid) => keys.push(format!("{keyid} {uid}")),
                    None => pending = Some(keyid.to_string()),
                }
            }
            Some(&"uid") => {
                if let Some(keyid) = pending.take() {
                    match fields.get(9).copied().filter(|uid| !uid.is_empty()) {
                        Some(uid) => keys.push(format!("{keyid} {uid}")),
                        None => keys.push(keyid),
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(keyid) = pending.take() {
        keys.push(keyid);
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_secret_keys_with_separate_uid_records() {
        let output = "\
sec:u:255:22:1234567890ABCDEF:1700000000:::u:::scESC:::+::ed25519:::0:
fpr:::::::::AAAA1234567890ABCDEF1234567890ABCDEF1234:
uid:u::::1700000000::HASH::Test User <test@example.com>::::::::::0:
sec:u:255:22:FEDCBA0987654321:1700000000:::u:::scESC:::+::ed25519:::0:
uid:u::::1700000000::HASH::Other User <other@example.com>::::::::::0:
";
        assert_eq!(
            parse_secret_keys(output),
            vec![
                "1234567890ABCDEF Test User <test@example.com>",
                "FEDCBA0987654321 Other User <other@example.com>",
            ]
        );
    }

    #[test]
    fn test_parse_secret_keys_with_uid_on_sec_record() {
        // Older gpg versions put the user id directly on the sec line
        let output =
            "sec:u:2048:1:1234567890ABCDEF:1700000000::::Test User <test@example.com>:::::::::\n";
        assert_eq!(
            parse_secret_keys(output),
            vec!["1234567890ABCDEF Test User <test@example.com>"]
        );
    }

    #[test]
    fn test_parse_secret_keys_without_uid_lists_keyid_alone() {
        let output = "sec:u:255:22:1234567890ABCDEF:1700000000:::u:::scESC:::+::ed25519:::0:\n";
        assert_eq!(parse_secret_keys(output), vec!["1234567890ABCDEF"]);
    }

    #[test]
    fn test_parse_secret_keys_empty_output() {
        assert!(parse_secret_keys("").is_empty());
    }
}
