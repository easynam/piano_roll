use std::sync::mpsc::{Receiver, SyncSender, channel, sync_channel};
use std::thread::sleep;
use std::time::Duration;

pub enum Command {
    Play,
    Stop,
}


pub struct Synth {
    recv: Receiver<Command>,
}

impl Synth {
    pub fn create() -> (SyncSender<Command>, Synth) {
        let (tx, rx) = sync_channel(64);
        (
            tx,
            Synth {
                recv: rx
            }
        )
    }

    pub fn run(self) {
        loop {
            sleep(Duration::from_millis(100));
            while let Ok(command) = self.recv.try_recv() {
                match command {
                    Command::Play => println!("play!"),
                    Command::Stop => println!("stop!"),
                }
            }
        }
    }
}