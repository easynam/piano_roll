mod audio_emitter;
mod controller;
mod redoxsynth;
mod mixer;
mod source;
mod player;

use std::{sync::{
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Mutex,
}, thread, time::Duration};

use crate::sequence::Sequence;

use self::{audio_emitter::AudioEmitter, player::Player, redoxsynth::RedoxSynthGenerator};

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
        (tx, Synth { recv: rx })
    }


    pub fn run(self, notes: Arc<Mutex<Sequence>>) {
        let mut emitter = AudioEmitter::new();
        let config = emitter.get_config();
        let (controller, source) = RedoxSynthGenerator::new(config.sample_rate.0 as f32, "gm.sf2").expect("redoxsynth init to succeed");
        let mut player = Player::new(
            200,
            notes.clone(),
            Box::new(controller)
        );
        emitter.start(Box::new(source));

        loop {
            while let Ok(command) = self.recv.try_recv() {
                match command {
                    Command::Play => player.play(emitter.get_sample_pos()),
                    Command::Stop => player.stop(),
                }
            }
            thread::sleep(Duration::from_millis(10));
            player.process(emitter.get_sample_pos() + 4800);
        }
    }
}
