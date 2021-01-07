mod audio_emitter;
mod controller;
mod effect;
mod mixer;
mod player;
mod redoxsynth;
mod source;

use std::{
    sync::{
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use iced::futures::channel::mpsc::{Receiver, Sender, channel};
use source::FxSource;

use crate::sequence::{Sequence, Pitch};

use self::{
    audio_emitter::AudioEmitter, effect::Delay, player::Player, redoxsynth::RedoxSynthGenerator,
};

#[derive(Debug, Clone)]
pub enum Command {
    SetNotes(Arc<Mutex<Sequence>>),
    Play,
    Stop,
    StartPreview(Pitch),
    StopPreview,
}

#[derive(Debug, Clone)]
pub enum Status {
    CommandChannel(Sender<Command>),
    PlaybackCursorUpdated(Option<f64>),
}

pub struct Synth {
    recv: Receiver<Command>,
    send: Sender<Status>,
}

impl Synth {
    pub fn create() -> (Receiver<Status>, Synth) {
        let (cmd_tx, cmd_rx) = channel(64);
        let (mut status_tx, status_rx) = channel(64);
        status_tx.try_send(Status::CommandChannel(cmd_tx));
        (status_rx, Synth { recv: cmd_rx, send: status_tx })
    }

    pub fn run(mut self) {
        let notes;

        loop {
            match self.recv.try_next() {
                Ok(Some(Command::SetNotes(n))) => {
                    notes = n;
                    break;
                }
                _ => {},
            }
        }

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
            while let Ok(Some(command)) = self.recv.try_next() {
                match command {
                    Command::Play => player.play(emitter.get_sample_pos()),
                    Command::Stop => player.stop(),
                    Command::StartPreview(pitch) => player.play_preview(pitch),
                    Command::StopPreview => player.stop_preview(),
                    Command::SetNotes(_) => panic!("SetNotes after init")
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
