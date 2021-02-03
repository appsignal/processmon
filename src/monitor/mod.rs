use std::io::{BufRead,BufReader};
use std::path::Path;
use std::process::{Command,Child,Stdio};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration,SystemTime};

use colored::*;

use crate::config::Config;
use crate::Result;

pub mod event_proxy;
use event_proxy::ChangeEvent;

pub struct Monitor {
    pub config: Config,
    pub receiver: Receiver<ChangeEvent>,
    pub running_processes: Vec<Child>,
    pub last_restart_at: Option<SystemTime>
}

impl Monitor {
    pub fn new(config: Config, receiver: Receiver<ChangeEvent>) -> Self {
        Self {
            config: config,
            receiver: receiver,
            running_processes: Vec::new(),
            last_restart_at: None
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Spawn the process
        self.spawn_processes()?;

        // Listen for change events
        loop {
            let event = self.receiver.recv()?;

            // See if we want to restart, only process events that were triggered
            // a bit later than the last restart.
            let restart = match self.last_restart_at {
                Some(last_restart_at) => match event.time.duration_since(last_restart_at) {
                    Ok(time) => time > Duration::from_secs(2),
                    Err(_) => false,
                },
                None => true
            };

            if restart {
                println!("Restarting");
                self.kill_running_processes()?;
                self.run_triggers(event.path.as_path())?;
                self.spawn_processes()?;
                self.last_restart_at = Some(SystemTime::now());
            }
        }
    }

    fn kill_running_processes(&mut self) -> Result<()> {
        for mut child in self.running_processes.drain(0..) {
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }

    fn spawn_processes(&mut self) -> Result<()> {
        for (name, process) in self.config.processes.iter() {
            println!("Starting process {} '{}'", name, process);

            // Create command
            let mut command = Command::new(&process.command);
            command.stderr(Stdio::piped());
            command.stdout(Stdio::piped());
            match process.args {
                Some(ref args) => {
                    command.args(args);
                },
                None => ()
            }

            // Spawn command
            let mut child = command.spawn()?;

            // Spawn threads that print output
            let stdout = BufReader::new(child.stdout.take().expect("Cannot take stdout"));
            let name_clone = name.to_owned();
            thread::spawn(move || {
                stdout.lines().for_each( |line| {
                    println!("{}: {}", name_clone.green(), line.unwrap());
                });
            });

            let stderr = BufReader::new(child.stderr.take().expect("Cannot take stdout"));
            let name_clone = name.to_owned();
            thread::spawn(move || {
                stderr.lines().for_each( |line| {
                    println!("{}: {}", name_clone.red(), line.unwrap());
                });
            });

            // Add to running processes
            self.running_processes.push(child);
        }
        Ok(())
    }

    fn run_triggers(&self, path: &Path) -> Result<()> {
        match self.config.triggers {
            Some(ref triggers) => {
                for (name, trigger) in triggers.iter() {
                    println!("Running trigger {} '{}'", name, trigger);
                    let mut command = Command::new(&trigger.command);
                    match trigger.args {
                        Some(ref args) => {
                            command.args(args);
                        },
                        None => ()
                    }
                    command.env("TRIGGER_PATH", path.to_string_lossy().to_string());
                    command.status()?;
                }
            },
            None => ()
        }
        Ok(())
    }
}
