# Processmon

Simple process monitor that can run and restart a process
if files are modified. To try it out:

```
cargo build
target/debug/processmon example/processmon.toml
```

Then save a file in the example directory to trigger a restart.

## Configuration

Processmon is configured by a toml file:

```
command = "command.sh"

paths_to_watch = ["project1", "project2"]

triggers = ["trigger.sh"]
```

`command` sets the command to run. Place one or more paths to watch
in `paths_to_watch`. `triggers` sets a list of commands that will run
before every restart of the main command.
