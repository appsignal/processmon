use std::path::PathBuf;
use std::time::SystemTime;

use notify::{Event, EventKind};

pub struct ChangeEvent {
    pub path: PathBuf,
    pub time: SystemTime,
}

impl ChangeEvent {
    /// Convert an event to a change event, returns
    /// none if we are ignoring this event.
    pub fn new(mut event: Event) -> Option<Self> {
        // Return none if this is not a modification
        match event.kind {
            EventKind::Access(_) => return None,
            EventKind::Any => return None,
            EventKind::Other => return None,
            _ => (),
        }
        // Return an event for the first path
        match event.paths.pop() {
            Some(path) if path.exists() => {
                let event = Self {
                    path: path,
                    time: SystemTime::now(),
                };
                Some(event)
            }
            Some(_) => None,
            None => None,
        }
    }
}
