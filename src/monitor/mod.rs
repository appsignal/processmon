use std::process::{Command,Child};
use std::sync::mpsc::Receiver;
use std::time::{Duration,SystemTime};

use crate::config::Config;

use crate::Result;

pub mod event_proxy;
use event_proxy::ChangeEvent;

pub struct Monitor {
    pub config: Config,
    pub receiver: Receiver<ChangeEvent>,
    pub command: Command,
    pub process: Option<Child>,
    pub last_restart_at: Option<SystemTime>
}

impl Monitor {
    pub fn new(config: Config, receiver: Receiver<ChangeEvent>) -> Self {
        let command = Command::new(config.command.clone());
        Self {
            config: config,
            receiver: receiver,
            command: command,
            process: None,
            last_restart_at: None
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Spawn the process
        self.spawn()?;

        // Listen for change events
        loop {
            let event = self.receiver.recv()?;

            // See if we want to restart, only process events a seconds
            // later than the last spawn.
            let restart = match self.last_restart_at {
                Some(last_restart_at) => event.time.duration_since(last_restart_at)? > Duration::from_secs(2),
                None => true
            };

            if restart {
                println!("Restarting {}", self.config.command);
                self.kill()?;
                self.spawn()?;
                self.last_restart_at = Some(SystemTime::now());
            }
        }
    }

    fn kill(&mut self) -> Result<()> {
        match self.process {
            Some(ref mut p) => p.kill()?,
            None => ()
        }
        self.process = None;
        Ok(())
    }

    fn spawn(&mut self) -> Result<()> {
        self.process = Some(self.command.spawn()?);
        Ok(())
    }
}
