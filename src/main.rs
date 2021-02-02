extern crate notify;
extern crate toml;

use std::env;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{Watcher, RecursiveMode, watcher};

mod config;
mod monitor;

use anyhow::Result;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();

    let current_dir = match env::current_dir() {
        Ok(d) => d.to_string_lossy().to_string(),
        Err(_) => "unknown".to_string()
    };

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

    println!("Starting processmon {} in {}: {:?}", VERSION, current_dir, config);

    // Verify all specified paths are present
    verify_path_present(&config.command, "command");
    match config.triggers {
        Some(ref triggers) => {
            for path in triggers.iter() {
                verify_path_present(&path, "trigger");
            }
        },
        None => ()
    }
    for path in config.paths_to_watch.iter() {
        verify_path_present(&path, "path to watch");
    }

    // Start watching paths
    let (watcher_sender, watcher_receiver) = channel();
    let mut watcher = watcher(watcher_sender, Duration::from_secs(1)).unwrap();
    for path in config.paths_to_watch.iter() {
        match watcher.watch(path, RecursiveMode::Recursive) {
            Ok(_) => (),
            Err(e) => panic!("Error adding path to watch '{}': {:?}", path, e)
        }
    }

    // Run event conversion proxy
    let (proxy_sender, proxy_receiver) = channel();
    monitor::event_proxy::run(watcher_receiver, proxy_sender);

    // Create and run monitor instance
    let mut monitor = monitor::Monitor::new(config, proxy_receiver);
    match monitor.run() {
        Ok(_) => (),
        Err(e) => panic!("Error running monitor: {:?}", e)
    }
}

fn verify_path_present(path: &str, name: &str) {
    if !Path::new(path).exists() {
        println!(
            "Path '{}' for {} not present",
            path,
            name
        );
        exit(1);
    }
}
