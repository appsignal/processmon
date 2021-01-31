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
    pub process: Option<Child>,
    pub last_restart_at: Option<SystemTime>
}

impl Monitor {
    pub fn new(config: Config, receiver: Receiver<ChangeEvent>) -> Self {
        Self {
            config: config,
            receiver: receiver,
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
                println!("Restarting '{}'", self.config.command);
                self.kill()?;
                self.run_triggers()?;
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
        let mut command = Command::new(&self.config.command);
        self.process = Some(command.spawn()?);
        Ok(())
    }

    fn run_triggers(&self) -> Result<()> {
        match self.config.triggers {
            Some(ref triggers) => {
                for trigger in triggers.iter() {
                    println!("Running trigger '{}'", trigger);
                    let mut command = Command::new(trigger);
                    command.spawn()?.wait()?;
                }
            },
            None => ()
        }
        Ok(())
    }
}
