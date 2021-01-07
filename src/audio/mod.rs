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

#[derive(Debug, Clone)]
pub enum Status {
    PlaybackCursorUpdated(Option<f64>),
}

pub struct Synth {
    recv: Receiver<Command>,
    send: SyncSender<Status>,
}

impl Synth {
    pub fn create() -> (SyncSender<Command>, Receiver<Status>, Synth) {
        let (cmd_tx, cmd_rx) = sync_channel(64);
        let (status_tx, status_rx) = sync_channel(64);
        (cmd_tx, status_rx, Synth { recv: cmd_rx, send: status_tx })
    }

    pub fn run(self, notes: Arc<Mutex<Sequence>>) {
        let mut cursor_pos = None;
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

            let sample_pos = emitter.get_sample_pos();
            player.process(sample_pos + 4800);

            let new_cursor_pos = player.get_position_at(sample_pos);
            if cursor_pos != new_cursor_pos {
                cursor_pos = new_cursor_pos;
                self.send.try_send(Status::PlaybackCursorUpdated(cursor_pos));
            }

            thread::sleep(Duration::from_millis(10));
        }
    }
}
