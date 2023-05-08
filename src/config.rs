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
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Config> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(toml::de::from_slice(&buffer)?)
    }

    pub fn in_debug_mode(&self) -> bool {
        self.debug_mode.unwrap_or(false)
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

        Config::from_file(&path).expect("Cannot load config");
    }
}
