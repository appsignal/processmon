# Processmon

Simple process monitor that can run and restart a process if files are
modified. To try it out:

```
cargo build
target/debug/processmon example/processmon.toml
```

Then save a file in the example directory to trigger a restart.
