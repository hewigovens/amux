# amux

[![CI](https://github.com/hewigovens/amux/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/amux/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/amux.svg)](https://crates.io/crates/amux)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`amux` (short for *agent multiplexer*) is a tiny CLI that keeps your local AI/code agents organised inside tmux. Launch, attach, detach, and remove agent sessions with one consistent interface that works across shells.

## Features

- Works with multiple sessions per agent (e.g. `--name review-123`)
- Understands built-in agent commands (`codex`, `claude`, `gemini`) out of the box
- Respects `CA_AGENT_CMD_<NAME>` environment overrides and `--cmd`/`--params`
- Provides status, attach, detach, start, and remove subcommands

## Installation

```bash
cargo install amux
```

Or build from source:

```bash
git clone https://github.com/hewigovens/amux
cd amux
cargo install --path .
```

## Usage

```bash
# List running agent sessions
amux status

# Launch the default codex agent (short flag or positional shortcut)
amux start codex
amux start -a codex

# Launch a second codex session with extra params
amux start codex -n review-123 -p "--mode review"

# Attach to an existing session (start it automatically if absent)
amux attach codex -n review-123 -s

# Detach all clients from a session
amux detach codex -n review-123

# Remove the tmux session entirely
amux rm codex -n review-123
```

### Custom agents

To register additional agents, set environment variables before running `amux`:

```bash
export CA_AGENT_CMD_myagent="my-agent-binary --flag foo"
```

You can also bypass configuration per command with `--cmd` and append extra arguments with `--params`.

## License

Licensed under the [MIT](LICENSE) license.
