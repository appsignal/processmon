extern crate notify;

use std::env;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = match args.get(1) {
        Some(c) => c.to_owned(),
        None => panic!("Please specify a command to run")
    };

    let path = match args.get(2) {
        Some(p) => p.to_owned(),
        None => panic!("Please specify a command to run")
    };

    println!("Starting process monitor for '{}', watching {}", command, path);

    // Start watching paths
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(path, RecursiveMode::Recursive).unwrap();

    let mut handle = match Command::new(command.clone()).spawn() {
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
                   handle = match Command::new(command.clone()).spawn() {
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
