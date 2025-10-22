use std::io;
use std::process::{Command, Stdio};

use crate::error::{bail, with_context, Result};

pub const SESSION_PREFIX: &str = "amux-";

#[derive(Debug)]
pub struct SessionDetail {
    pub session_name: String,
    pub agent: String,
    pub name: Option<String>,
    pub client_count: usize,
    pub pane_command: Option<String>,
}

pub fn session_name(agent: &str, name: Option<&str>) -> String {
    match name {
        Some(name) => format!("{SESSION_PREFIX}{agent}--{name}"),
        None => format!("{SESSION_PREFIX}{agent}"),
    }
}

pub fn list_sessions() -> Result<Vec<SessionDetail>> {
    let output = tmux_command()
        .arg("list-sessions")
        .arg("-F")
        .arg("#S")
        .output();

    let raw_sessions = match output {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("no server running") {
                    Vec::new()
                } else {
                    return bail(format!(
                        "tmux list-sessions exited with status {}",
                        output.status
                    ));
                }
            }
        }
        Err(err) => return Err(tmux_invoke_error(err)),
    };

    let mut sessions = Vec::new();
    for session in raw_sessions {
        if let Some((agent, name)) = parse_session_name(&session) {
            let client_count = client_count(&session)?;
            let pane_command = current_command(&session)?;
            sessions.push(SessionDetail {
                session_name: session,
                agent,
                name,
                client_count,
                pane_command,
            });
        }
    }

    Ok(sessions)
}

pub fn new_session(session: &str, command_tokens: &[String]) -> Result<()> {
    let mut cmd = tmux_command();
    cmd.arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(session)
        .arg("--")
        .args(command_tokens);
    let status = cmd.status().map_err(tmux_invoke_error)?;
    if status.success() {
        Ok(())
    } else {
        bail(format!("tmux new-session exited with status {status}"))
    }
}

pub fn kill_session(session: &str) -> Result<()> {
    let status = tmux_command()
        .arg("kill-session")
        .arg("-t")
        .arg(session)
        .status()
        .map_err(tmux_invoke_error)?;
    if status.success() {
        Ok(())
    } else {
        bail(format!("tmux kill-session exited with status {status}"))
    }
}

pub fn has_session(session: &str) -> Result<bool> {
    let status = tmux_command()
        .arg("has-session")
        .arg("-t")
        .arg(session)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(tmux_invoke_error)?;
    Ok(status.success())
}

pub fn client_count(session: &str) -> Result<usize> {
    let output = tmux_command()
        .arg("list-clients")
        .arg("-t")
        .arg(session)
        .output()
        .map_err(tmux_invoke_error)?;

    if output.status.success() {
        let count = String::from_utf8_lossy(&output.stdout).lines().count();
        Ok(count)
    } else if output.stderr.is_empty() {
        Ok(0)
    } else {
        bail(format!(
            "tmux list-clients exited with status {}",
            output.status
        ))
    }
}

pub fn attach_session(session: &str) -> Result<()> {
    let status = tmux_command()
        .arg("attach-session")
        .arg("-t")
        .arg(session)
        .status()
        .map_err(tmux_invoke_error)?;

    if status.success() {
        Ok(())
    } else {
        bail(format!("tmux attach-session exited with status {status}"))
    }
}

pub fn detach_clients(session: &str) -> Result<()> {
    let status = tmux_command()
        .arg("detach-client")
        .arg("-s")
        .arg(session)
        .status()
        .map_err(tmux_invoke_error)?;

    if status.success() {
        Ok(())
    } else {
        bail(format!("tmux detach-client exited with status {status}"))
    }
}

fn current_command(session: &str) -> Result<Option<String>> {
    let output = tmux_command()
        .arg("display-message")
        .arg("-p")
        .arg("-t")
        .arg(session)
        .arg("#{pane_current_command}")
        .output()
        .map_err(tmux_invoke_error)?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(text))
        }
    } else {
        Ok(None)
    }
}

fn parse_session_name(session: &str) -> Option<(String, Option<String>)> {
    let rest = session.strip_prefix(SESSION_PREFIX)?;
    let mut parts = rest.splitn(2, "--");
    let agent = parts.next()?.to_string();
    let name = parts.next().map(|s| s.to_string());
    Some((agent, name))
}

fn tmux_command() -> Command {
    let mut cmd = Command::new("tmux");
    cmd.env("TMUX", "");
    cmd
}

fn tmux_invoke_error(err: io::Error) -> crate::error::DynError {
    if err.kind() == io::ErrorKind::NotFound {
        // Provide actionable guidance when tmux is not installed
        crate::error::fail(
            "tmux not found. Please install tmux and try again.\n\
             - macOS: brew install tmux\n\
             - Debian/Ubuntu: sudo apt-get update && sudo apt-get install tmux\n\
             - Nix: nix-env -iA nixpkgs.tmux\n\
             See: https://github.com/tmux/tmux/wiki/Installing",
        )
    } else {
        with_context(err, "failed to invoke tmux")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_name_without_suffix() {
        let name = session_name("codex", None);
        assert_eq!(name, format!("{SESSION_PREFIX}codex"));
    }

    #[test]
    fn session_name_with_suffix() {
        let name = session_name("codex", Some("review"));
        assert_eq!(name, format!("{SESSION_PREFIX}codex--review"));
    }

    #[test]
    fn parse_session_name_splits_agent_and_name() {
        let parsed = parse_session_name(&format!("{SESSION_PREFIX}codex--review"))
            .expect("session name should parse");
        assert_eq!(parsed.0, "codex");
        assert_eq!(parsed.1.as_deref(), Some("review"));
    }

    #[test]
    fn parse_session_name_handles_agent_only() {
        let parsed =
            parse_session_name(&format!("{SESSION_PREFIX}codex")).expect("agent-only should parse");
        assert_eq!(parsed.0, "codex");
        assert!(parsed.1.is_none());
    }

    #[test]
    fn parse_session_name_returns_none_for_unexpected_prefix() {
        assert!(parse_session_name("other-codex").is_none());
    }
}
