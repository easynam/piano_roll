use audio::Status;
use iced::{Application, Column, Element, Error, Row, Settings};
use iced_native::{Container, Button, Text};
use widgets::piano_roll::{PianoRoll, PianoRollSettings};
use std::{fmt::Debug, sync::{Arc, Mutex, mpsc::Receiver}};
use crate::scroll_zoom::{ScrollZoomState, ScrollScaleAxisChange, ScrollScaleAxis};
use crate::sequence::{SequenceChange, Sequence, update_sequence};
use crate::widgets::scroll_bar::{ScrollZoomBarState, Orientation, ScrollZoomBar};
use widgets::piano_roll;
use iced_native::widget::button;
use std::sync::mpsc::SyncSender;
use crate::audio::{Command, Synth};
use std::thread;
use crate::widgets::piano_roll::Action;

mod audio;
mod sequence;
mod widgets;
mod scroll_zoom;
mod helpers;

pub fn main() -> Result<(), Error> {
    App::run(Settings::default())
}

struct App {
    piano_roll: piano_roll::PianoRollState,
    scroll_zoom: ScrollZoomState,
    scroll_bar: ScrollZoomBarState,
    scroll_bar_2: ScrollZoomBarState,
    notes: Arc<Mutex<Sequence>>,
    settings: PianoRollSettings,
    play_button: button::State,
    stop_button: button::State,
    synth_channel: SyncSender<Command>,
    status_channel: Receiver<Status>,
}

#[derive(Debug, Clone)]
enum Message {
    Sequence(SequenceChange),
    Scroll(ScrollScaleAxisChange),
    Scroll2(ScrollScaleAxisChange),
    PianoRoll(Action),
    SynthCommand(Command),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        let (synth_channel, status_channel, synth) = Synth::create();

        let notes = Arc::new(Mutex::new(vec!()));

        {
            let notes = notes.clone();
            thread::spawn(move|| {
                synth.run(notes);
            });
        }

        (
            App {
                piano_roll: piano_roll::PianoRollState::new(),
                scroll_zoom:ScrollZoomState {
                    x: ScrollScaleAxis::new(0.0,32.0*32.0, 0.0, 32.0*32.0*4.0),
                    y: ScrollScaleAxis::new(-1.5*200.0, 3.0*200.0, -4.0*200.0, 8.0*200.0),
                },
                scroll_bar: ScrollZoomBarState::new(),
                scroll_bar_2: ScrollZoomBarState::new(),
                notes,
                settings: PianoRollSettings::default(),
                play_button: button::State::new(),
                stop_button: button::State::new(),
                synth_channel,
                status_channel,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        "wow".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
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
            Message::SynthCommand(command) => {
                self.synth_channel.try_send(command);
            },
            Message::PianoRoll(action) => {
                self.piano_roll.action = action;
            }
        }
        iced::Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .push(Row::new()
                .push(Container::new(
                    PianoRoll::new(&mut self.piano_roll, self.notes.as_ref(), Message::Sequence, Message::PianoRoll, Message::SynthCommand, &self.scroll_zoom, &self.settings))
                    .max_width(800)
                    .max_height(600))
                .push(Container::new(
                    ScrollZoomBar::new(
                        &mut self.scroll_bar_2, &self.scroll_zoom.y, Message::Scroll2, Orientation::Vertical, false
                    ))
                    .max_height(600)
                    .max_width(20)
                )
            )
            .push(Container::new(
                ScrollZoomBar::new(
                    &mut self.scroll_bar, &self.scroll_zoom.x, Message::Scroll, Orientation::Horizontal, false
                ))
                .max_width(800)
                .max_height(20)
            )
            .push(Row::new()
                .push(Button::new(&mut self.play_button, Text::new("Play"))
                    .on_press(Message::SynthCommand(Command::Play)))
                .push(Button::new(&mut self.stop_button, Text::new("Stop"))
                    .on_press(Message::SynthCommand(Command::Stop)))
            )
            .padding(40)
            .into()
    }
}
