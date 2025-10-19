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
    let mut keys = vec![format!("CA_AGENT_CMD_{agent}")];
    let upper = agent.to_ascii_uppercase();
    if upper != agent {
        keys.push(format!("CA_AGENT_CMD_{upper}"));
    }

    for key in keys {
        if let Ok(val) = env::var(&key) {
            if !val.trim().is_empty() {
                return Some(val);
            }
        }
    }
    None
}
