use std::process::{Command,Child};
use std::sync::mpsc::Receiver;

use crate::config::Config;

use crate::Result;

pub mod event_proxy;
use event_proxy::ChangeEvent;

pub struct Monitor {
    pub config: Config,
    pub receiver: Receiver<ChangeEvent>,
    pub command: Command,
    pub process: Option<Child>
}

impl Monitor {
    pub fn new(config: Config, receiver: Receiver<ChangeEvent>) -> Self {
        let command = Command::new(config.command.clone());
        Self {
            config: config,
            receiver: receiver,
            command: command,
            process: None
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Spawn the process
        self.spawn()?;

        // Listen for change events
        loop {
            let event = self.receiver.recv()?;
            println!("Received change event at {:?}", event.time);
            self.kill()?;
            self.spawn()?;
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
