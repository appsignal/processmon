extern crate notify;
extern crate toml;

use std::env;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::channel;
use std::time::Duration;

use colored::Colorize;
use notify::{watcher, RecursiveMode, Watcher};

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
    let (watcher_sender, watcher_receiver) = channel();
    let mut watcher = watcher(watcher_sender, Duration::from_secs(1)).unwrap();
    for path_config in config.paths_to_watch.iter() {
        match watcher.watch(&path_config.path, RecursiveMode::Recursive) {
            Ok(_) => (),
            Err(e) => panic!("Error adding path to watch '{}': {:?}", path_config.path, e),
        }
    }

    // Run event conversion proxy
    let (proxy_sender, proxy_receiver) = channel();
    monitor::event_proxy::run(watcher_receiver, proxy_sender);

    // Create and run monitor instance
    let mut monitor = monitor::Monitor::new(config, proxy_receiver);
    match monitor.run() {
        Ok(_) => (),
        Err(e) => panic!("Error running monitor: {:?}", e),
    }
}
