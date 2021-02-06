use std::path::Path;

use crate::config::Config;

pub struct Ignore {

}

impl Ignore {
    pub fn new(config: &Config) -> Self {
        Self {

        }
    }

    pub fn should_ignore(&self, path: &Path) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {


}
