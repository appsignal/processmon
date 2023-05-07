extern crate notify;
extern crate toml;

use std::env;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::channel;

use colored::Colorize;
use notify::{RecursiveMode, Watcher};

mod change_event;
mod config;
mod monitor;

use anyhow::Result;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

enum Command {
    Start,
    Connect,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // See what we are doing
    let command = match args.get(1).map(|arg| arg.as_str()) {
        Some("start") => Command::Start,
        Some("connect") => Command::Connect,
        _ => {
            eprintln!("Use start or connect as the first argument");
            exit(1);
        }
    };

    // Load config
    let config = match config::Config::from_file(Path::new("processmon.toml")) {
        Ok(c) => c,
        Err(e) => {
            println!("Cannot load config from processmon.toml: {:?}", e);
            exit(1);
        }
    };

    match command {
        Command::Start => start(config),
        Command::Connect => connect(config),
    }
}

fn start(config: config::Config) {
    println!("Starting {} {}", "processmon".bold(), VERSION);

    // Start watching paths
    let (monitor_sender, monitor_receiver) = channel();
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => match change_event::ChangeEvent::new(event) {
            Some(event) => {
                monitor_sender
                    .send(event)
                    .expect("Cannot send event to channel");
            }
            None => (),
        },
        Err(e) => panic!("Watcher error: {:?}", e),
    })
    .expect("Cannot create watcher");
    for path_config in config.paths_to_watch.iter() {
        match watcher.watch(Path::new(&path_config.path), RecursiveMode::Recursive) {
            Ok(_) => (),
            Err(e) => panic!("Error adding path to watch '{}': {:?}", path_config.path, e),
        }
    }

    // Create and run monitor instance
    let mut monitor = monitor::Monitor::new(config, monitor_receiver);
    match monitor.run() {
        Ok(_) => (),
        Err(e) => panic!("Error running monitor: {:?}", e),
    }
}

fn connect(config: Config) {}
