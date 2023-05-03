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

fn main() {
    let args: Vec<String> = env::args().collect();

    // Load config
    let config_file_path = match args.get(1) {
        Some(p) => p.to_owned(),
        None => {
            println!("Please specify a config file to use");
            exit(1);
        }
    };
    let config = match config::Config::from_file(Path::new(&config_file_path)) {
        Ok(c) => c,
        Err(e) => {
            println!("Cannot load config from {}: {:?}", config_file_path, e);
            exit(1);
        }
    };

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
