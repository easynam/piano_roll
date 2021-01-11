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
    Pause,
    Stop,
    Seek(i32),
    StartPreview(Pitch),
    StopPreview,
}

#[derive(Debug, Clone)]
pub enum Status {
    CommandChannel(Sender<Command>),
    PlaybackCursorUpdated(i32),
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

        let mut sample_pos = 0;
        let mut cursor_pos = 0;
        let mut emitter = AudioEmitter::new();
        let config = emitter.get_config();
        let (controller, source) = RedoxSynthGenerator::new(config.sample_rate.0 as f32, "gm.sf2")
            .expect("redoxsynth init to succeed");
        let mut player = Player::new(200, notes.clone(), Box::new(controller), 4800);
        let delay = Delay::new(10000, 0.0, 1.0, 0.75);
        let fxsource = FxSource::new(Box::new(source), vec![Box::new(delay)]);
        let mut samples_receiver = emitter.start(Box::new(fxsource));

        let mut start_cursor = 0;

        loop {
            while let Ok(Some(command)) = self.recv.try_next() {
                match command {
                    Command::Play => {
                        player.play(sample_pos);
                    },
                    Command::Stop => {
                        player.pause();
                        player.seek(sample_pos, start_cursor);
                    },
                    Command::Pause => {
                        player.pause();
                    },
                    Command::Seek(seek_pos) => {
                        player.seek(sample_pos, seek_pos);
                    },
                    Command::StartPreview(pitch) => player.play_preview(pitch),
                    Command::StopPreview => player.stop_preview(),
                    Command::SetNotes(_) => panic!("SetNotes after init")
                }
            }

            let new_cursor_pos = player.get_position();
            if cursor_pos != new_cursor_pos {
                cursor_pos = new_cursor_pos;
                self.send.try_send(Status::PlaybackCursorUpdated(cursor_pos));
            }

            while let Ok(Some(samples)) = samples_receiver.try_next() {
                player.process(samples);
                sample_pos += samples;
            }

            thread::sleep(Duration::from_millis(10));
        }
    }
}
