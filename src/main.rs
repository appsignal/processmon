extern crate notify;
extern crate toml;

use std::env;
use std::fs;
use std::io::{stdin, stdout, Read, Write};
use std::net::UdpSocket;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::channel;
use std::thread;

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
        Command::Connect => connect(config, args),
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

fn connect(config: config::Config, args: Vec<String>) {
    // Get the process we are conecting to
    let process = match args.get(2).map(|arg| arg.as_str()) {
        Some(p) => p,
        None => {
            eprintln!("Pick a process");
            exit(1);
        }
    };

    // Get the command config for the process we are connecting to
    let command_config = match config.processes.get(process) {
        Some(c) => c,
        None => {
            eprintln!("Process {} not in config", process);
            exit(1);
        }
    };

    // Get ports
    let our_port = command_config.connect_port.expect("No connect port set");
    let process_port = command_config.process_port.expect("No process port set");

    println!(
        "Connecting to {} on {}, type away...",
        process, process_port
    );

    // Create a socket
    let socket =
        UdpSocket::bind(format!("127.0.0.1:{}", our_port)).expect("Could not start socket");

    // Read and print incoming data from the socket
    let read_socket = socket.try_clone().unwrap();
    let mut read_buffer = [0; 65_536];
    let mut tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .unwrap();
    thread::spawn(move || loop {
        let (bytes_read, _source) = read_socket.recv_from(&mut read_buffer).unwrap();
        tty.write_all(&read_buffer[0..bytes_read]).unwrap();
    });

    // Send stdin to the socket
    let mut buffer = [0; 65_536];
    let mut stdin = stdin();
    let address = format!("127.0.0.1:{}", process_port);
    loop {
        let bytes_read = stdin.read(&mut buffer[..]).unwrap();
        socket
            .send_to(&buffer[..bytes_read], &address)
            .expect("Failed sending");
    }
}
