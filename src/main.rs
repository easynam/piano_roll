use std::{fmt::Debug, sync::{Arc, Mutex}};
use std::thread;

use iced::{Application, Column, Element, Error, futures::{self, channel::mpsc::Sender}, Row, Settings, Subscription};
use iced_native::{Button, Text, Space, Length};
use iced_native::widget::button;

use audio::Status;
use widgets::piano_roll::{PianoRoll, PianoRollSettings};
use widgets::piano_roll;

use crate::audio::{Command, Synth, PlaybackState};
use crate::scroll_zoom::{ScrollScaleAxis, ScrollScaleAxisChange, ScrollZoomState};
use crate::sequence::{Sequence, SequenceChange};
use crate::widgets::piano_roll::{PianoRollMessage};
use crate::widgets::scroll_bar::{Orientation, ScrollZoomBar, ScrollZoomBarState};
use crate::widgets::timeline::{Timeline, TimelineState};

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
    timeline: TimelineState,
    notes: Arc<Mutex<Sequence>>,
    settings: PianoRollSettings,
    play_button: button::State,
    stop_button: button::State,
    synth_channel: Option<Sender<Command>>,
    playback_state: PlaybackState,
}

#[derive(Debug, Clone)]
enum Message {
    Sequence(SequenceChange),
    Scroll(ScrollScaleAxisChange),
    Scroll2(ScrollScaleAxisChange),
    PianoRoll(PianoRollMessage),
    SynthCommand(Command),
    SynthStatus(Status),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        (
            App {
                piano_roll: piano_roll::PianoRollState::new(),
                scroll_zoom:ScrollZoomState {
                    x: ScrollScaleAxis::new(0.0,32.0*32.0, 0.0, 32.0*32.0*4.0),
                    y: ScrollScaleAxis::new(-1.5, 3.0, -4.0, 8.0),
                },
                scroll_bar: ScrollZoomBarState::new(),
                scroll_bar_2: ScrollZoomBarState::new(),
                timeline: TimelineState::new(),
                notes: Arc::new(Mutex::new(Sequence::new())),
                settings: PianoRollSettings::default(),
                play_button: button::State::new(),
                stop_button: button::State::new(),
                synth_channel: None,
                playback_state: PlaybackState::new(),
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
                notes.update_sequence(change);
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
                if let Some(channel) = self.synth_channel.as_mut() {
                    channel.try_send(command);
                }
            },
            Message::PianoRoll(action) => {
                self.piano_roll.on_event(action, &*self.notes.lock().unwrap());
            }
            Message::SynthStatus(status) => match status {
                Status::CommandChannel(mut channel) => {
                    channel.try_send(Command::SetNotes(self.notes.clone()));
                    self.synth_channel = Some(channel);
                },
                Status::PlaybackStateUpdated(state) => {
                    self.playback_state = state;
                }
            }
        }
        iced::Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .push(Row::new()
                .push(Timeline::new(
                    &self.scroll_zoom.x,
                    &self.settings,
                    Message::SynthCommand,
                    &mut self.timeline,
                    &self.playback_state,
                ))
                .push(Space::new(Length::Units(20), Length::Shrink))
            )
            .push(Row::new()
                .push(PianoRoll::new(
                    &mut self.piano_roll,
                    self.notes.as_ref(),
                    Message::Sequence,
                    Message::PianoRoll,
                    Message::SynthCommand,
                    &self.scroll_zoom,
                    &self.settings,
                    &self.playback_state,
                ))
                .push(ScrollZoomBar::new(
                    &mut self.scroll_bar_2,
                    &self.scroll_zoom.y,
                    Message::Scroll2,
                    Orientation::Vertical,
                    false,
                ))
                .height(Length::Fill)
            )
            .push(Row::new()
                .push(ScrollZoomBar::new(
                    &mut self.scroll_bar,
                    &self.scroll_zoom.x,
                    Message::Scroll,
                    Orientation::Horizontal,
                    true,
                ))
                .push(Space::new(Length::Units(20), Length::Shrink))
            )
            .push(Row::new()
                .push(Button::new(&mut self.play_button, Text::new("Play"))
                    .on_press(Message::SynthCommand(Command::Play)))
                .push(Button::new(&mut self.stop_button, Text::new("Stop"))
                    .on_press(Message::SynthCommand(Command::Stop)))
                .height(Length::Units(50))
            )
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::from_recipe(SynthThread("main synth thread")).map(|x| Message::SynthStatus(x))
    }
}

struct SynthThread(&'static str);

impl<H, I> iced_native::subscription::Recipe<H, I> for SynthThread
where
    H: std::hash::Hasher,
{
    type Output = Status;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.0.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        let (status_channel, synth) = Synth::create();
        thread::spawn(move|| {
            synth.run();
        });
        Box::pin(status_channel)
    }
}
