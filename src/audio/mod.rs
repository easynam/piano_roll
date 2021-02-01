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
pub enum SynthCommand {
    SetNotes(Arc<Mutex<Sequence>>),
    Play,
    Pause,
    Stop,
    Seek(i32),
    SetLoop(Option<(i32, i32)>),
    StartPreview(Pitch),
    StopPreview,
}

#[derive(Debug, Clone)]
pub enum Status {
    CommandChannel(Sender<SynthCommand>),
    PlaybackStateUpdated(PlaybackState),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackState {
    pub playback_cursor: i32,
    pub playback_start_cursor: i32,
    pub playing: bool,
    pub looping: Option<(i32,i32)>
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            playback_cursor: 0,
            playback_start_cursor: 0,
            playing: false,
            looping: None,
        }
    }
}

pub struct Synth {
    recv: Receiver<SynthCommand>,
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
                Ok(Some(SynthCommand::SetNotes(n))) => {
                    notes = n;
                    break;
                }
                _ => {},
            }
        }

        let mut sample_pos = 0;
        let mut playback_state = PlaybackState::new();
        let mut last_playback_state = playback_state.clone();
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
                    SynthCommand::Play => {
                        player.play(sample_pos);
                        playback_state.playing = true;
                    },
                    SynthCommand::Stop => {
                        player.pause();
                        player.seek(sample_pos, start_cursor);
                        playback_state.playing = false;
                    },
                    SynthCommand::Pause => {
                        player.pause();
                        playback_state.playing = false;
                    },
                    SynthCommand::Seek(seek_pos) => {
                        player.seek(sample_pos, seek_pos);
                        start_cursor = seek_pos;
                        playback_state.playback_start_cursor = seek_pos;
                    },
                    SynthCommand::StartPreview(pitch) => player.play_preview(pitch),
                    SynthCommand::StopPreview => player.stop_preview(),
                    SynthCommand::SetNotes(_) => panic!("SetNotes after init"),
                    SynthCommand::SetLoop(looping) => {
                        player.set_loop(looping);
                        playback_state.looping = looping;
                    }
                }
            }

            playback_state.playback_cursor = player.get_position();
            if playback_state != last_playback_state {
                last_playback_state = playback_state.clone();
                self.send.try_send(Status::PlaybackStateUpdated(playback_state.clone()));
            }

            while let Ok(Some(samples)) = samples_receiver.try_next() {
                player.process(samples);
                sample_pos += samples;
            }

            thread::sleep(Duration::from_millis(10));
        }
    }
}
