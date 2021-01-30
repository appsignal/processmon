extern crate notify;
extern crate toml;

use std::env;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};

pub mod config;

use anyhow::Result;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config_file_path = match args.get(1) {
        Some(p) => p.to_owned(),
        None => panic!("Please specify a config file to use")
    };

    let config = match config::Config::from_file(Path::new(&config_file_path)) {
        Ok(c) => c,
        Err(e) => panic!("Cannot load config from {}: {:?}", config_file_path, e)
    };

    println!("Starting processmon: {:?}", config);

    // Start watching paths
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    for path in config.paths_to_watch.iter() {
        watcher.watch(path, RecursiveMode::Recursive).unwrap();
    }

    let mut handle = match Command::new(config.command.clone()).spawn() {
        Ok(h) => h,
        Err(e) => panic!("Error launching command: {:?}", e)
    };

    loop {
        match rx.recv() {
           Ok(event) => match event {
               DebouncedEvent::Create(_) |
                   DebouncedEvent::Write(_) |
                   DebouncedEvent::Chmod(_) |
                   DebouncedEvent::Remove(_) |
                   DebouncedEvent::Rename(_, _) => {
                   println!("Got event: {:?}", event);
                   handle.kill().unwrap();
                   handle = match Command::new(config.command.clone()).spawn() {
                       Ok(h) => h,
                       Err(e) => panic!("Error launching command: {:?}", e)
                   };
               },
               _ => ()
           },
           Err(e) => println!("Watch error: {:?}", e),
        }
    }
}
