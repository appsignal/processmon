use std::sync::mpsc::{Receiver,Sender};
use std::thread;
use std::time::SystemTime;

use notify::DebouncedEvent;

pub struct ChangeEvent {
    pub time: SystemTime
}

impl ChangeEvent {
    pub fn new() -> Self {
        Self { time: SystemTime::now() }
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
    match event {
        DebouncedEvent::Create(_) |
            DebouncedEvent::Write(_) |
            DebouncedEvent::Chmod(_) |
            DebouncedEvent::Remove(_) |
            DebouncedEvent::Rename(_, _) => Some(ChangeEvent::new()),
       _ => None
    }
}
