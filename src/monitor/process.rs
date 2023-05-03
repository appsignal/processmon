use std::collections::HashMap;
use std::io::{BufRead, BufReader};
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
) -> Result<Child> {
    // Create command
    let mut command = Command::new(&config.command);

    // Pipe stdout and err so we can capture it and
    // print it with a prefix.
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

    // Spawn command
    let mut child = command.spawn()?;

    // Spawn threads that print stdout and stderr
    let stdout = BufReader::new(child.stdout.take().expect("Cannot take stdout"));
    let color_clone = color.to_owned();
    let prefix_clone = prefix.to_owned();
    thread::spawn(move || {
        stdout.lines().for_each(|line| {
            println!(
                "{}: {}",
                prefix_clone.color(color_clone.clone()),
                line.unwrap()
            );
        });
    });

    let stderr = BufReader::new(child.stderr.take().expect("Cannot take stdout"));
    let color_clone = color.to_owned();
    let prefix_clone = prefix.to_owned();
    thread::spawn(move || {
        stderr.lines().for_each(|line| {
            println!(
                "{}: {}",
                prefix_clone.color(color_clone.clone()),
                line.unwrap()
            );
        });
    });

    Ok(child)
}
