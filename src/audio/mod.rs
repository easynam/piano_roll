mod audio_emitter;
mod controller;
mod effect;
mod mixer;
mod player;
mod redoxsynth;
mod source;

use std::{
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use source::FxSource;

use crate::sequence::{Sequence, Pitch};

use self::{
    audio_emitter::AudioEmitter, effect::Delay, player::Player, redoxsynth::RedoxSynthGenerator,
};

#[derive(Debug, Clone)]
pub enum Command {
    Play,
    Stop,
    StartPreview(Pitch),
    StopPreview,
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
        let (controller, source) = RedoxSynthGenerator::new(config.sample_rate.0 as f32, "gm.sf2")
            .expect("redoxsynth init to succeed");
        let mut player = Player::new(200, notes.clone(), Box::new(controller));
        let delay = Delay::new(10000, 0.0, 1.0, 0.75);
        let fxsource = FxSource::new(Box::new(source), vec![Box::new(delay)]);
        emitter.start(Box::new(fxsource));

        loop {
            while let Ok(command) = self.recv.try_recv() {
                match command {
                    Command::Play => player.play(emitter.get_sample_pos()),
                    Command::Stop => player.stop(),
                    Command::StartPreview(pitch) => player.play_preview(pitch),
                    Command::StopPreview => player.stop_preview(),
                }
            }
            thread::sleep(Duration::from_millis(10));
            player.process(emitter.get_sample_pos() + 4800);
        }
    }
}
