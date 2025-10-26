use std::collections::BTreeSet;
use std::env;

use crate::error::{bail, with_context, Result};

#[derive(Clone, Copy)]
struct DefaultAgent {
    name: &'static str,
    command: &'static [&'static str],
    description: &'static str,
}

const DEFAULT_AGENTS: &[DefaultAgent] = &[
    DefaultAgent {
        name: "codex",
        command: &["codex"],
        description: "Codex CLI",
    },
    DefaultAgent {
        name: "claude",
        command: &["claude"],
        description: "Claude CLI",
    },
    DefaultAgent {
        name: "gemini",
        command: &["gemini"],
        description: "Gemini CLI",
    },
    DefaultAgent {
        name: "opencode",
        command: &["opencode"],
        description: "OpenCode CLI",
    },
];

pub fn resolve_agent_command(agent: &str, command_override: Option<&str>) -> Result<Vec<String>> {
    if let Some(raw) = command_override {
        return parse_tokens("command override", raw);
    }

    if let Some(raw) = lookup_env_command(agent) {
        return parse_tokens("environment override", &raw);
    }

    if let Some(default) = default_agent(agent) {
        return Ok(default.command.iter().map(|s| (*s).to_string()).collect());
    }

    bail(format!(
        "no command configured for agent '{agent}'. Set CA_AGENT_CMD_{agent} or provide --cmd explicitly."
    ))
}

pub fn configured_agents() -> Vec<String> {
    let mut names = BTreeSet::new();
    for default in DEFAULT_AGENTS {
        names.insert(default.name.to_string());
    }
    for (key, _) in env::vars() {
        if let Some(agent) = key.strip_prefix("CA_AGENT_CMD_") {
            names.insert(agent.to_ascii_lowercase());
        }
    }
    names.into_iter().collect()
}

pub fn agent_description(name: &str) -> Option<&'static str> {
    default_agent(name).map(|agent| agent.description)
}

pub fn is_default_agent(name: &str) -> bool {
    default_agent(name).is_some()
}

pub fn parse_tokens(origin: &str, raw: &str) -> Result<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return bail(format!("{origin} is empty"));
    }
    let tokens = shell_words::split(trimmed)
        .map_err(|err| with_context(err, format!("failed to parse {origin}")))?;
    if tokens.is_empty() {
        return bail(format!("{origin} produced no tokens"));
    }
    Ok(tokens)
}

fn default_agent(name: &str) -> Option<DefaultAgent> {
    DEFAULT_AGENTS
        .iter()
        .copied()
        .find(|agent| agent.name == name)
}

fn lookup_env_command(agent: &str) -> Option<String> {
    // Try the agent name as-is first
    let key = format!("CA_AGENT_CMD_{agent}");
    if let Ok(val) = env::var(&key) {
        if !val.trim().is_empty() {
            return Some(val);
        }
    }

    // Try uppercase variant if different
    let upper = agent.to_ascii_uppercase();
    if upper != agent {
        let key = format!("CA_AGENT_CMD_{upper}");
        if let Ok(val) = env::var(&key) {
            if !val.trim().is_empty() {
                return Some(val);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_agent_command_defaults_to_builtin() {
        let command = resolve_agent_command("codex", None).expect("default agent should resolve");
        assert_eq!(command, vec!["codex"]);
    }

    #[test]
    fn resolve_agent_command_honors_override() {
        let command =
            resolve_agent_command("codex", Some("custom --flag")).expect("override should parse");
        assert_eq!(command, vec!["custom", "--flag"]);
    }

    #[test]
    fn parse_tokens_trims_and_splits() {
        let tokens = parse_tokens("origin", "run --mode review").expect("tokens expected");
        assert_eq!(tokens, vec!["run", "--mode", "review"]);
    }

    #[test]
    fn parse_tokens_rejects_empty_input() {
        let err = parse_tokens("origin", "   ").expect_err("empty input should error");
        assert!(err.to_string().contains("origin is empty"));
    }

    #[test]
    fn configured_agents_include_defaults() {
        let agents = configured_agents();
        assert!(agents.contains(&"codex".to_string()));
        assert!(agents.contains(&"claude".to_string()));
        assert!(agents.contains(&"gemini".to_string()));
        assert!(agents.contains(&"opencode".to_string()));
    }
}
