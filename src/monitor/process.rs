use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::thread;

use colored::*;

use crate::config::CommandConfig;
use crate::Result;

pub fn spawn(
    prefix: &str,
    color: &str,
    config: &CommandConfig,
    env: Option<HashMap<String, String>>,
    port: Option<i32>,
) -> Result<Child> {
    // Create command
    let mut command = Command::new(&config.command);

    // Pipe std in, out and err so we can capture it
    // and print/write to it
    command.stdin(Stdio::piped());
    command.stderr(Stdio::piped());
    command.stdout(Stdio::piped());

    // Set args if specified
    match config.args {
        Some(ref args) => {
            command.args(args);
        }
        None => (),
    }

    // Set working dir if specified
    match config.working_dir {
        Some(ref working_dir) => {
            command.current_dir(working_dir);
        }
        None => (),
    }

    // Set env vars if specified
    match config.env {
        Some(ref env) => {
            for (key, var) in env.iter() {
                command.env(key.to_uppercase(), &var);
            }
        }
        None => (),
    }

    // Set additional env vars if supplied
    match env {
        Some(ref env) => {
            command.envs(env);
        }
        None => (),
    }

    // Create udp socket to be able to connect from the outside
    let socket = match port {
        Some(port) => {
            println!("Binding to {}", port);
            Some(UdpSocket::bind(format!("127.0.0.1:{}", port))?)
        }
        None => None,
    };

    // Spawn command
    let mut child = command.spawn()?;

    // Spawn thread to read from udp socket if we have one
    let read_socket = match socket {
        Some(ref socket) => Some(socket.try_clone().unwrap()),
        None => None,
    };
    match read_socket {
        Some(read_socket) => {
            let mut stdin = child.stdin.take().expect("Cannot not take stdin");
            let mut buffer = [0; 65_536];
            thread::spawn(move || loop {
                match read_socket.recv_from(&mut buffer) {
                    Ok((bytes_read, source)) => {
                        // Write the read bytes to the proceses stdin
                        stdin.write(&buffer[0..bytes_read]).unwrap();
                    }
                    // TODO: store source to write to later
                    Err(err) => {
                        eprintln!("Error reading from socket: {}", err);
                        continue;
                    }
                };
            });
        }
        None => (),
    }

    // Spawn threads that print stdout and stderr
    let stdout = BufReader::new(child.stdout.take().expect("Cannot take stdout"));
    let color_clone = color.to_owned();
    let prefix_clone = prefix.to_owned();
    thread::spawn(move || {
        stdout.lines().for_each(|line| {
            let line = line.unwrap();
            println!(
                "{}: {}",
                prefix_clone.color(color_clone.clone()),
                line
            );
            match socket {
                Some(ref socket) => {
                    socket.send_to(
                        line.as_bytes(),
                        "127.0.0.1:41102"
                    ).unwrap();
                },
                None => (),
            }
        });
    });

    let stderr = BufReader::new(child.stderr.take().expect("Cannot take stderr"));
    let color_clone = color.to_owned();
    let prefix_clone = prefix.to_owned();
    thread::spawn(move || {
        stderr.lines().for_each(|line| {
            let line = line.expect("Could not read line from process stdout");
            // Print to our stdout
            println!(
                "{} stderr: {}",
                prefix_clone.color(color_clone.clone()),
                line
            );
            // Emit on UDP socket if present
            //match socket {
            //    Some(ref socket) => match socket.write(line.as_bytes()) {
            //        Ok(_) => (),
            //        Err(err) => eprintln!("Error writing to socket: {}", err),
            //    },
            //    None => ()
            //}
        });
    });

    Ok(child)
}
