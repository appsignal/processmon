# Processmon

Process monitor that can run trigger scripts and subsequently
restarts the processes it monitors when files on specified paths
are modified. This can be very useful when running a development
environment. Especially in a Docker container that's slow to restart.

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
paths_to_watch = [
  "code"
]

[processes]

[processes.process1]
command = "sh"
args = ["process1.sh"]

[processes.process2]
command = "./process2.sh"

[triggers]

[triggers.trigger]
command = "sh"
args = ["../trigger.sh"]
working_dir = "code"

[triggers.trigger.env]
content_for_file = "Triggered"
```

### Paths to watch

Place one or more paths to watch in `paths_to_watch`. Any changes here
will restart monitored processes and run any configured triggers.

### Processes

Specify processes to run and monitor in `processes`.

### Triggers

Specify triggers to run before restart in `triggers`. When running a trigger
the env var `TRIGGER_PATH` will be set with the path that triggered the
restart.

### Command configuration

Both processes and triggers share the same configuration options:

 * `command`: Command to run.
 * `args`: List of argument to supply to the command (optional)
 * `working_dir`: Working directory that the command will run in (optional)
 * `env`: Environment variables for command (optional)
