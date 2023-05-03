use std::collections::HashMap;
use std::path::Path;
use std::process::Child;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime};

use crate::change_event::ChangeEvent;
use crate::config::Config;
use crate::Result;

mod ignore;
mod process;

const COLORS: &[&str] = &[
    "bright green",
    "bright blue",
    "yellow",
    "magenta",
    "bright cyan",
];

pub struct Monitor {
    config: Config,
    receiver: Receiver<ChangeEvent>,
    running_processes: Vec<Child>,
    last_restart_at: Option<SystemTime>,
    ignore: ignore::Ignore,
}

impl Monitor {
    pub fn new(config: Config, receiver: Receiver<ChangeEvent>) -> Self {
        let ignore = ignore::Ignore::new(&config);
        Self {
            config: config,
            ignore: ignore,
            receiver: receiver,
            running_processes: Vec::new(),
            last_restart_at: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Spawn the process
        self.spawn_processes()?;

        // Listen for change events
        loop {
            let event = self.receiver.recv()?;

            // See if this is an ignored path
            if self.ignore.should_ignore(&event.path) {
                if self.config.in_debug_mode() {
                    println!("Ignoring changed path {}", event.path.to_string_lossy());
                }
                continue;
            }

            // See if we want to restart, only process events that were triggered
            // a bit later than the last restart.
            let restart = match self.last_restart_at {
                Some(last_restart_at) => match event.time.duration_since(last_restart_at) {
                    Ok(time) => time > Duration::from_secs(2),
                    Err(_) => false,
                },
                None => true,
            };

            if restart {
                println!("Restarting ({} changed)", event.path.to_string_lossy());
                self.kill_running_processes()?;
                self.run_triggers(event.path.as_path())?;
                self.spawn_processes()?;
                self.last_restart_at = Some(SystemTime::now());
            }
        }
    }

    fn kill_running_processes(&mut self) -> Result<()> {
        if self.config.in_debug_mode() {
            println!(
                "Killing {} running process(es)",
                self.running_processes.len()
            );
        }
        for mut child in self.running_processes.drain(0..) {
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }

    fn spawn_processes(&mut self) -> Result<()> {
        let mut color_i = 0;
        for (name, command_config) in self.config.processes.iter() {
            if self.config.in_debug_mode() {
                println!("Starting process {} '{}'", name, command_config);
            }

            // Spawn child process
            let child = process::spawn(&name, COLORS[color_i], command_config, None)?;

            // Add to running processes
            self.running_processes.push(child);

            // Determine next color
            if color_i == COLORS.len() - 1 {
                color_i = 0;
            } else {
                color_i += 1;
            }
        }
        Ok(())
    }

    fn run_triggers(&self, path: &Path) -> Result<()> {
        match self.config.triggers {
            Some(ref triggers) => {
                // Prepare env
                let mut env = HashMap::new();
                env.insert(
                    "TRIGGER_PATH".to_owned(),
                    path.to_string_lossy().to_string(),
                );

                for (name, command_config) in triggers.iter() {
                    if self.config.in_debug_mode() {
                        println!("Running trigger {} '{}'", name, command_config);
                    }

                    // Spawn child process
                    let mut child =
                        process::spawn(&name, "green", command_config, Some(env.clone()))?;

                    // Wait for it to finish
                    child.wait()?;
                }
            }
            None if self.config.in_debug_mode() => println!("No triggers configured"),
            None => (),
        }
        Ok(())
    }
}
