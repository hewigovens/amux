use std::collections::BTreeMap;

use clap::{Parser, Subcommand};

use crate::agents;
use crate::error::{bail, with_context, Result};
use crate::tmux::{self, SessionDetail};

#[derive(Parser, Debug)]
#[command(
    name = "amux",
    version,
    about = "tmux session manager for local code agents",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show available commands and configured agents
    Help,
    /// Show the current state of configured agent sessions
    Status {
        /// Optional agent name to filter results
        agent: Option<String>,
    },
    /// Alias for `status`
    List {
        /// Optional agent name to filter results
        agent: Option<String>,
    },
    /// Launch an agent inside tmux (use --force to restart)
    Start {
        /// Agent identifier (alphanumeric, '-' or '_')
        #[arg(short = 'a', long, value_name = "AGENT", conflicts_with = "agent_pos")]
        agent: Option<String>,
        /// Optional positional shortcut for default agents
        #[arg(value_name = "AGENT", conflicts_with = "agent")]
        agent_pos: Option<String>,
        /// Optional session name to allow multiple sessions per agent
        #[arg(short = 'n', long)]
        name: Option<String>,
        /// Replace the configured command with a custom one (parsed like a shell command)
        #[arg(short = 'c', long = "cmd", value_name = "COMMAND")]
        command_override: Option<String>,
        /// Additional parameters appended to the agent command (parsed like a shell command)
        #[arg(short = 'p', long, value_name = "PARAMS")]
        params: Option<String>,
        /// Kill an existing session before starting
        #[arg(short = 'f', long)]
        force: bool,
    },
    /// Remove the tmux session for an agent
    Rm {
        /// Agent identifier (alphanumeric, '-' or '_')
        #[arg(short = 'a', long, value_name = "AGENT", conflicts_with = "agent_pos")]
        agent: Option<String>,
        /// Optional positional shortcut for default agents
        #[arg(value_name = "AGENT", conflicts_with = "agent")]
        agent_pos: Option<String>,
        /// Optional session name if the agent has multiple sessions
        #[arg(short = 'n', long)]
        name: Option<String>,
    },
    /// Attach to an agent's tmux session
    Attach {
        /// Agent identifier (alphanumeric, '-' or '_')
        #[arg(short = 'a', long, value_name = "AGENT", conflicts_with = "agent_pos")]
        agent: Option<String>,
        /// Optional positional shortcut for default agents
        #[arg(value_name = "AGENT", conflicts_with = "agent")]
        agent_pos: Option<String>,
        /// Optional session name if the agent has multiple sessions
        #[arg(short = 'n', long)]
        name: Option<String>,
        /// Launch the agent if the session does not exist
        #[arg(short = 's', long)]
        start: bool,
    },
    /// Detach all clients from an agent's tmux session
    Detach {
        /// Agent identifier (alphanumeric, '-' or '_')
        #[arg(short = 'a', long, value_name = "AGENT", conflicts_with = "agent_pos")]
        agent: Option<String>,
        /// Optional positional shortcut for default agents
        #[arg(value_name = "AGENT", conflicts_with = "agent")]
        agent_pos: Option<String>,
        /// Optional session name if the agent has multiple sessions
        #[arg(short = 'n', long)]
        name: Option<String>,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Help => {
            print_help();
        }
        Commands::Status { agent } | Commands::List { agent } => {
            handle_status(agent)?;
        }
        Commands::Start {
            agent,
            agent_pos,
            name,
            command_override,
            params,
            force,
        } => {
            let agent = resolve_agent_input(agent, agent_pos, "start")?;
            handle_start(
                &agent,
                name.as_deref(),
                command_override.as_deref(),
                params.as_deref(),
                force,
            )?;
        }
        Commands::Rm {
            agent,
            agent_pos,
            name,
        } => {
            let agent = resolve_agent_input(agent, agent_pos, "rm")?;
            handle_rm(&agent, name.as_deref())?;
        }
        Commands::Attach {
            agent,
            agent_pos,
            name,
            start,
        } => {
            let agent = resolve_agent_input(agent, agent_pos, "attach")?;
            handle_attach(&agent, name.as_deref(), start)?;
        }
        Commands::Detach {
            agent,
            agent_pos,
            name,
        } => {
            let agent = resolve_agent_input(agent, agent_pos, "detach")?;
            handle_detach(&agent, name.as_deref())?;
        }
    }

    Ok(())
}

fn handle_start(
    agent: &str,
    session_name: Option<&str>,
    command_override: Option<&str>,
    params: Option<&str>,
    force: bool,
) -> Result<()> {
    ensure_valid_identifier("agent", agent)?;
    if let Some(name) = session_name {
        ensure_valid_identifier("session name", name)?;
    }

    let session_id = tmux::session_name(agent, session_name);
    let mut command_tokens = agents::resolve_agent_command(agent, command_override)?;

    if let Some(extra) = params {
        let mut extra_tokens = agents::parse_tokens("params", extra)?;
        command_tokens.append(&mut extra_tokens);
    }

    if command_tokens.is_empty() {
        return bail(format!("resolved command for '{agent}' is empty"));
    }

    if tmux::has_session(&session_id)? {
        if force {
            tmux::kill_session(&session_id)?;
        } else {
            println!("{agent}: session '{session_id}' already running (use --force to restart)");
            return Ok(());
        }
    }

    tmux::new_session(&session_id, &command_tokens)
        .map_err(|err| with_context(err, format!("failed to start agent '{agent}'")))?;

    println!("{agent}: started in session '{session_id}'");
    Ok(())
}

fn handle_rm(agent: &str, session_name: Option<&str>) -> Result<()> {
    ensure_valid_identifier("agent", agent)?;
    if let Some(name) = session_name {
        ensure_valid_identifier("session name", name)?;
    }

    let session_id = tmux::session_name(agent, session_name);

    if !tmux::has_session(&session_id)? {
        println!("{agent}: no active session (looked for '{session_id}')");
        return Ok(());
    }

    tmux::kill_session(&session_id)?;

    println!("{agent}: removed session '{session_id}'");
    Ok(())
}

fn handle_attach(agent: &str, session_name: Option<&str>, start: bool) -> Result<()> {
    ensure_valid_identifier("agent", agent)?;
    if let Some(name) = session_name {
        ensure_valid_identifier("session name", name)?;
    }

    let session_id = tmux::session_name(agent, session_name);

    if !tmux::has_session(&session_id)? {
        if start {
            handle_start(agent, session_name, None, None, false)?;
        } else {
            println!(
                "{agent}: no active session (looked for '{session_id}'); pass --start to launch"
            );
            return Ok(());
        }
    }

    tmux::attach_session(&session_id)?;
    Ok(())
}

fn handle_detach(agent: &str, session_name: Option<&str>) -> Result<()> {
    ensure_valid_identifier("agent", agent)?;
    if let Some(name) = session_name {
        ensure_valid_identifier("session name", name)?;
    }

    let session_id = tmux::session_name(agent, session_name);

    if !tmux::has_session(&session_id)? {
        println!("{agent}: no active session (looked for '{session_id}')");
        return Ok(());
    }

    let clients = tmux::client_count(&session_id)?;
    if clients == 0 {
        println!("{agent}: no clients to detach");
        return Ok(());
    }

    tmux::detach_clients(&session_id)?;
    println!("{agent}: detached clients from '{session_id}'");
    Ok(())
}

fn handle_status(agent_filter: Option<String>) -> Result<()> {
    let sessions = tmux::list_sessions()?;

    let mut sessions_by_agent: BTreeMap<&str, Vec<&SessionDetail>> = BTreeMap::new();
    for session in &sessions {
        sessions_by_agent
            .entry(session.agent.as_str())
            .or_default()
            .push(session);
    }

    if let Some(agent) = agent_filter {
        ensure_valid_identifier("agent", &agent)?;
        if let Some(agent_sessions) = sessions_by_agent.get(agent.as_str()) {
            print_agent_sessions(&agent, agent_sessions);
        } else {
            println!("{agent}: no sessions running.");
        }
        return Ok(());
    }

    if sessions.is_empty() {
        println!("No agent sessions are running.");
        return Ok(());
    }

    for (agent, agent_sessions) in &sessions_by_agent {
        print_agent_sessions(agent, agent_sessions);
    }

    Ok(())
}

fn print_agent_sessions(agent: &str, sessions: &[&SessionDetail]) {
    let mut entries: Vec<&SessionDetail> = sessions.to_vec();
    entries.sort_by(|a, b| a.session_name.cmp(&b.session_name));

    for session in entries {
        let name_part = session
            .name
            .as_ref()
            .map(|name| format!(", name '{name}'"))
            .unwrap_or_default();
        let pane = session.pane_command.as_deref().unwrap_or("-");
        println!(
            "{agent}: running (session '{}'{}, clients: {}, command: {})",
            session.session_name, name_part, session.client_count, pane
        );
    }
}

fn print_help() {
    println!("amux – tmux session manager for local code agents");
    println!();
    println!("Commands:");
    println!("  amux help                Show this overview");
    println!("  amux status [agent]      Show agent session state");
    println!("  amux start [-a NAME|NAME] [-n SESSION] [-p \"...\"] [-f]");
    println!("                         Launch an agent session (use -f/--force to restart)");
    println!("  amux rm [-a NAME|NAME] [-n SESSION]");
    println!("                         Remove the agent's tmux session");
    println!("  amux attach [-a NAME|NAME] [-n SESSION] [-s]");
    println!("                         Attach to an agent session (use -s/--start to launch)");
    println!("  amux detach [-a NAME|NAME] [-n SESSION]");
    println!("                         Detach all clients from an agent session");
    println!();

    let agents = agents::configured_agents();
    if agents.is_empty() {
        println!("No agents configured.");
        return;
    }

    println!("Configured agents:");
    for agent in agents {
        if let Some(description) = agents::agent_description(&agent) {
            println!("  {agent:<12} {description}");
        } else {
            println!("  {agent}");
        }
    }
}

fn ensure_valid_identifier(kind: &str, value: &str) -> Result<()> {
    let is_valid = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if is_valid {
        Ok(())
    } else {
        bail(format!(
            "{kind} '{value}' contains invalid characters (allowed: a-z, A-Z, 0-9, '-', '_')"
        ))
    }
}

fn resolve_agent_input(
    agent_flag: Option<String>,
    agent_pos: Option<String>,
    command: &str,
) -> Result<String> {
    if let Some(agent) = agent_flag {
        return Ok(agent);
    }

    if let Some(agent) = agent_pos {
        if agents::is_default_agent(&agent) {
            return Ok(agent);
        }
        return bail(format!(
            "{command}: '{agent}' is not a default agent; use --agent/-a to specify custom agents"
        ));
    }

    bail(format!(
        "{command}: agent name required; supply a default agent shortcut or --agent/-a <name>"
    ))
}
