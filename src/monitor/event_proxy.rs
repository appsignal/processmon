use std::path::PathBuf;
use std::sync::mpsc::{Receiver,Sender};
use std::thread;
use std::time::SystemTime;

use notify::DebouncedEvent;

pub struct ChangeEvent {
    pub path: PathBuf,
    pub time: SystemTime
}

impl ChangeEvent {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path,
            time: SystemTime::now()
        }
    }
}

/// Run a proxy that listens to debounced events
/// and converts them to change events.
pub fn run(
    receiver: Receiver<DebouncedEvent>,
    sender: Sender<ChangeEvent>
) {
    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(event) => match convert(event) {
                    Some(converted) => match sender.send(converted) {
                        Ok(_) => (),
                        Err(e) => panic!("Error sending change event: {:?}", e)
                    },
                    None => ()
                },
                Err(e) => panic!("Error listening for debounced events: {:?}", e)
            }
        }
    });
}

/// Convert to change event, return none if this one should
/// be ignored.
fn convert(event: DebouncedEvent) -> Option<ChangeEvent> {
    // Extract path
    let path = match event {
        DebouncedEvent::Create(p) => p,
        DebouncedEvent::Write(p) => p,
        DebouncedEvent::Chmod(p) => p,
        DebouncedEvent::Remove(p) => p,
        DebouncedEvent::Rename(_, p) => p,
       _ => return None
    };

    // Only return it if the path exists
    if path.exists() {
        Some(ChangeEvent::new(path))
    } else {
        None
    }
}
