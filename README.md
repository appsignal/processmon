# Processmon

Process monitor that can run trigger scripts and subsequently
restarts the process when files on specified paths are modified.
This can be very useful when running a development environment in
a Docker container.

To try it out:

```
cargo build
cd example
../target/debug/processmon processmon.toml
```

Then save a file in the `code` directory to trigger a restart.

## Installation

Make sure you have a recent version of Rust installed, then run:

```
cargo install processmon
```

## Configuration

Processmon is configured by a toml file:

```
command = "./command.sh"

paths_to_watch = ["code", "dependency_code"]

triggers = ["./trigger.sh"]
```

`command` sets the command to run. Place one or more paths to watch
in `paths_to_watch`. `triggers` sets a list of commands that will run
before every restart of the main command.
