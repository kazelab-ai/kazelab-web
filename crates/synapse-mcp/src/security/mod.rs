//! Tool Call Sandbox Sandboxing and Command Injection Isolation Policy.

use std::collections::HashSet;

pub struct ToolSecurityPolicy {
    blocked_commands: HashSet<String>,
    blocked_path_traversals: HashSet<String>,
}

impl ToolSecurityPolicy {
    pub fn new() -> Self {
        let mut blocked_commands = HashSet::new();
        blocked_commands.insert("rm -rf /".into());
        blocked_commands.insert("chmod -R 777".into());
        blocked_commands.insert("mkfs".into());
        blocked_commands.insert("curl | bash".into());

        let mut blocked_path_traversals = HashSet::new();
        blocked_path_traversals.insert("../..".into());
        blocked_path_traversals.insert("/etc/passwd".into());
        blocked_path_traversals.insert("/root".into());

        Self {
            blocked_commands,
            blocked_path_traversals,
        }
    }

    pub fn validate_command(&self, cmd: &str) -> Result<(), &'static str> {
        let lower = cmd.to_lowercase();
        for blocked in &self.blocked_commands {
            if lower.contains(blocked) {
                return Err("Security Violation: Disallowed destructive command pattern detected");
            }
        }
        for path in &self.blocked_path_traversals {
            if lower.contains(path) {
                return Err("Security Violation: Path traversal attack pattern detected");
            }
        }
        Ok(())
    }
}
