use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use toml;

use crate::Result;

#[derive(Debug,Clone,Deserialize)]
pub struct CommandConfig {
    pub command: String,
    pub args: Option<Vec<String>>
}

impl fmt::Display for CommandConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.args {
            Some(ref args) => write!(f, "{} {}", self.command, args.join(" ")),
            None => write!(f, "{}", self.command)
        }
    }
}

#[derive(Debug,Clone,Deserialize)]
pub struct Config {
    pub paths_to_watch: Vec<String>,
    pub processes: HashMap<String, CommandConfig>,
    pub triggers: Option<HashMap<String, CommandConfig>>
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Config> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(toml::de::from_slice(&buffer)?)
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use super::*;

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
