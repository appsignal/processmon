use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use toml;

use crate::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct PathConfig {
    /// Path that will be watched
    pub path: String,
    /// Sub-paths within the path to ignore
    pub ignore: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommandConfig {
    /// Command to run
    pub command: String,
    /// Arguments to supply to the command
    pub args: Option<Vec<String>>,
    /// Working directory that the command will run in
    pub working_dir: Option<String>,
    /// Environment variables for command
    pub env: Option<BTreeMap<String, String>>,
    /// UDP port to use on the process side
    pub process_port: Option<i32>,
    /// UDP port to use on the connect side
    pub connect_port: Option<i32>,
}

impl fmt::Display for CommandConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.args {
            Some(ref args) => write!(f, "{} {}", self.command, args.join(" ")),
            None => write!(f, "{}", self.command),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Paths that will be watched
    pub paths_to_watch: Vec<PathConfig>,
    /// Proceses to run and monitor
    pub processes: BTreeMap<String, CommandConfig>,
    /// Triggers that will run when changes are detected on the paths
    pub triggers: Option<BTreeMap<String, CommandConfig>>,
    /// Enable to get more detailed output
    pub debug_mode: Option<bool>,
    /// Start of the port range to use
    pub port_range_start: Option<i32>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Config> {
        // Read the config file
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let mut config: Self = toml::de::from_slice(&buffer)?;
        // Set port config
        let mut port = config.port_range_start();
        for (_name, command_config) in config.processes.iter_mut() {
            command_config.process_port = Some(port);
            port += 1;
            command_config.connect_port = Some(port);
            port += 1;
        }
        // Return it
        Ok(config)
    }

    pub fn in_debug_mode(&self) -> bool {
        self.debug_mode.unwrap_or(false)
    }

    pub fn port_range_start(&self) -> i32 {
        self.port_range_start.unwrap_or(40_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;

    #[test]
    fn test_config_from_file_invalid_path() {
        assert!(Config::from_file(Path::new("nonsense")).is_err());
    }

    #[test]
    fn test_config_from_file() {
        let path = current_dir()
            .expect("Could not get current dir")
            .join("example/processmon.toml");

        let config = Config::from_file(&path).expect("Cannot load config");
        assert_eq!(40_000, config.port_range_start());

        let mut iter = config.processes.iter();
        let (name, command_config) = iter.next().unwrap();
        assert_eq!("process1", name);
        assert_eq!(Some(40_000), command_config.process_port);
        assert_eq!(Some(40_001), command_config.connect_port);

        let (name, command_config) = iter.next().unwrap();
        assert_eq!("process2", name);
        assert_eq!(Some(40_002), command_config.process_port);
        assert_eq!(Some(40_003), command_config.connect_port);

        let (name, command_config) = iter.next().unwrap();
        assert_eq!("process3", name);
        assert_eq!(Some(40_004), command_config.process_port);
        assert_eq!(Some(40_005), command_config.connect_port);
    }
}
