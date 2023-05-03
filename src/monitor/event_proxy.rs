use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::SystemTime;

use notify::{Event, EventKind};

pub struct ChangeEvent {
    pub path: PathBuf,
    pub time: SystemTime,
}

impl ChangeEvent {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path,
            time: SystemTime::now(),
        }
    }
}

/// Run a proxy that listens to debounced events
/// and converts them to change events.
pub fn run(receiver: Receiver<Event>, sender: Sender<ChangeEvent>) {
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => match convert(event) {
                Some(converted) => match sender.send(converted) {
                    Ok(_) => (),
                    Err(e) => panic!("Error sending change event: {:?}", e),
                },
                None => (),
            },
            Err(e) => panic!("Error listening for debounced events: {:?}", e),
        }
    });
}

/// Convert to change event, return none if this one should
/// be ignored.
fn convert(mut event: Event) -> Option<ChangeEvent> {
    // Return none if this is not a modification
    match event.kind {
        EventKind::Access(_) => return None,
        EventKind::Any => return None,
        EventKind::Other => return None,
        _ => (),
    }

    // Return an event for the first path
    match event.paths.pop() {
        Some(path) if path.exists() => Some(ChangeEvent::new(path)),
        Some(_) => None,
        None => None,
    }
}
