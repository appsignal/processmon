use std::env;
use std::path::{Path, PathBuf};

use crate::config::Config;

pub struct Ignore {
    paths: Vec<PathBuf>,
}

impl Ignore {
    pub fn new(config: &Config) -> Self {
        let mut paths = Vec::new();

        // Collect full list of paths to ignore
        for path_config in config.paths_to_watch.iter() {
            match path_config.ignore {
                Some(ref ignore) => {
                    let path = Path::new(&path_config.path).to_path_buf();
                    // See if this is a root path, or if we should
                    // prefix it with the working directory
                    let path = if path.has_root() {
                        path
                    } else {
                        let current_dir = env::current_dir().expect("Could not get current dir");
                        current_dir.join(path).to_path_buf()
                    };
                    for ignore_path in ignore.iter() {
                        let ignore_path = Path::new(ignore_path);
                        let combined_path = path.join(ignore_path).to_path_buf();
                        paths.push(combined_path);
                    }
                }
                None => (),
            }
        }

        Self { paths: paths }
    }

    pub fn should_ignore(&self, path: &Path) -> bool {
        for ignore_path in self.paths.iter() {
            if path.starts_with(ignore_path) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_ignore_with_root_path() {
        let config = helpers::mock_config("/path", "tmp");
        let ignore = Ignore::new(&config);

        // Don't ignore these
        assert!(!ignore.should_ignore(&Path::new("something")));
        assert!(!ignore.should_ignore(&Path::new("/path/something")));

        // Ignore these
        assert!(ignore.should_ignore(&Path::new("/path/tmp")));
        assert!(ignore.should_ignore(&Path::new("/path/tmp/some_file.txt")));
    }

    #[test]
    fn test_ignore_wth_relative_path() {
        let current_dir = env::current_dir().unwrap();

        let config = helpers::mock_config("path", "tmp");
        let ignore = Ignore::new(&config);

        // Don't ignore these
        assert!(!ignore.should_ignore(&Path::new("something")));
        assert!(!ignore.should_ignore(&current_dir.join("path/something")));

        // Ignore these
        assert!(ignore.should_ignore(&current_dir.join("path/tmp")));
        assert!(ignore.should_ignore(&current_dir.join("path/tmp/some_file.txt")));
    }

    mod helpers {
        use super::*;
        use crate::config::PathConfig;

        pub fn mock_config(path: &str, ignore: &str) -> Config {
            let paths_to_watch = vec![PathConfig {
                path: path.to_owned(),
                ignore: Some(vec![ignore.to_owned()]),
            }];
            Config {
                paths_to_watch: paths_to_watch,
                processes: HashMap::new(),
                triggers: None,
                debug_mode: None,
            }
        }
    }
}
