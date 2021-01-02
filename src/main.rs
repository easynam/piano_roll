use iced::{Element, Settings, Sandbox, Column, Row};
use iced_native::{Container, Button, Text};
use widgets::piano_roll::{PianoRoll, PianoRollSettings};
use std::{fmt::Debug, sync::{Arc, Mutex}};
use crate::scroll_zoom::{ScrollZoomState, ScrollScaleAxisChange};
use crate::sequence::{SequenceChange, Sequence, update_sequence};
use crate::widgets::scroll_bar::{ScrollZoomBarState, ScrollZoomBarX};
use widgets::piano_roll;
use iced_native::widget::button;
use std::sync::mpsc::SyncSender;
use crate::audio::{Command, Synth};
use std::thread;

mod audio;
mod sequence;
mod widgets;
mod scroll_zoom;
mod helpers;

pub fn main() {
    App::run(Settings::default())
}

struct App {
    piano_roll_1: piano_roll::PianoRollState,
    scroll_zoom: ScrollZoomState,
    scroll_bar: ScrollZoomBarState,
    scroll_bar_2: ScrollZoomBarState,
    notes: Arc<Mutex<Sequence>>,
    settings: PianoRollSettings,
    play_button: button::State,
    stop_button: button::State,
    synth_channel: SyncSender<Command>,
}

#[derive(Debug, Clone)]
enum Message {
    Sequence(SequenceChange),
    Scroll(ScrollScaleAxisChange),
    Scroll2(ScrollScaleAxisChange),
    Play,
    Stop,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        let (synth_channel, synth) = Synth::create();

        let notes = Arc::new(Mutex::new(vec!()));

        {
            let notes = notes.clone();
            thread::spawn(move|| {
                synth.run(notes);
            });
        }

        App {
            piano_roll_1: piano_roll::PianoRollState::new(),
            scroll_zoom: Default::default(),
            scroll_bar: ScrollZoomBarState::new(),
            scroll_bar_2: ScrollZoomBarState::new(),
            notes,
            settings: PianoRollSettings::default(),
            play_button: button::State::new(),
            stop_button: button::State::new(),
            synth_channel,
        }
    }

    fn title(&self) -> String {
        "wow".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Sequence(change) => {
                let mut notes = self.notes.lock().unwrap();
                update_sequence(&mut notes, change)
            },
            Message::Scroll(scroll) => match scroll {
                ScrollScaleAxisChange::Left(new_pos) => {
                    self.scroll_zoom.x.view_start = new_pos
                },
                ScrollScaleAxisChange::Right(new_pos) => {
                    self.scroll_zoom.x.view_end = new_pos
                },
                _ => {}
            },
            Message::Scroll2(scroll) => match scroll {
                ScrollScaleAxisChange::Left(new_pos) => {
                    self.scroll_zoom.y.view_start = new_pos
                },
                ScrollScaleAxisChange::Right(new_pos) => {
                    self.scroll_zoom.y.view_end = new_pos
                },
                _ => {}
            },
            Message::Play => {
                self.synth_channel.try_send(Command::Play);
            },
            Message::Stop => {
                self.synth_channel.try_send(Command::Stop);
            },
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .push(Container::new(
                PianoRoll::new(&mut self.piano_roll_1, self.notes.as_ref(), Message::Sequence, &self.scroll_zoom, &self.settings))
                .max_height(600)
            )
            .push(Container::new(
                ScrollZoomBarX::new(
                    &mut self.scroll_bar, &self.scroll_zoom.x, Message::Scroll, false
                )).max_height(20)
            )
            .push(Container::new(
                ScrollZoomBarX::new(
                    &mut self.scroll_bar_2, &self.scroll_zoom.y, Message::Scroll2, false
                )).max_height(20)
            )
            .push(Row::new()
                .push(Button::new(&mut self.play_button, Text::new("Play"))
                    .on_press(Message::Play))
                .push(Button::new(&mut self.stop_button, Text::new("Stop"))
                    .on_press(Message::Stop))
            )
            .padding(20)
            .into()
    }
}
