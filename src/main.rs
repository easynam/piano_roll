use std::{fmt::Debug, sync::{Arc, Mutex}};
use std::thread;

use iced::{Application, Column, Element, Error, futures::{self, channel::mpsc::Sender}, Row, Settings, Subscription};
use iced_native::{Button, Text, Length, subscription, keyboard, Event};
use iced_native::widget::button;

use audio::Status;
use widgets::piano_roll::PianoRollSettings;

use crate::audio::{SynthCommand, Synth, PlaybackState};
use crate::sequence::{Sequence, SequenceChange};
use iced::keyboard::KeyCode;
use crate::widgets::sequence_editor::{SequenceEditor, SequenceEditorSelfMessage, SequenceEditorMessage};

mod audio;
mod sequence;
mod widgets;
mod scroll_zoom;
mod helpers;

pub fn main() -> Result<(), Error> {
    App::run(Settings::default())
}

struct App {
    notes: Arc<Mutex<Sequence>>,
    settings: PianoRollSettings,
    play_button: button::State,
    stop_button: button::State,
    synth_channel: Option<Sender<SynthCommand>>,
    playback_state: PlaybackState,
    sequence_editor: SequenceEditor,
}

#[derive(Debug, Clone)]
enum Message {
    Sequence(SequenceChange),
    SynthCommand(SynthCommand),
    SynthStatus(Status),
    PlayOrStop,
    SequenceEditorMessage(SequenceEditorSelfMessage),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        (
            App {
                notes: Arc::new(Mutex::new(Sequence::new())),
                settings: PianoRollSettings::default(),
                play_button: button::State::new(),
                stop_button: button::State::new(),
                synth_channel: None,
                playback_state: PlaybackState::new(),
                sequence_editor: Default::default(),
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
            Message::SynthCommand(command) => {
                if let Some(channel) = self.synth_channel.as_mut() {
                    channel.try_send(command);
                }
            },
            Message::SynthStatus(status) => match status {
                Status::CommandChannel(mut channel) => {
                    channel.try_send(SynthCommand::SetNotes(self.notes.clone()));
                    self.synth_channel = Some(channel);
                },
                Status::PlaybackStateUpdated(state) => {
                    self.playback_state = state;
                }
            }
            Message::PlayOrStop => {
                if let Some(channel) = self.synth_channel.as_mut() {
                    match self.playback_state.playing {
                        true => channel.try_send(SynthCommand::Stop),
                        false => channel.try_send(SynthCommand::Play)
                    };
                }
            }
            Message::SequenceEditorMessage(message) => {
                self.sequence_editor.update(message, &self.notes);
            }
        }
        iced::Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            Subscription::from_recipe(SynthThread("main synth thread")).map(|x| Message::SynthStatus(x)),
            subscription::events_with(|event, _status| {
                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => match key_code {
                        KeyCode::Space => Some(Message::PlayOrStop),
                        _ => None
                    }
                    _ => None
                }
            })
        ])
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .push(self.sequence_editor.view(
                &self.notes, &self.settings, &self.playback_state
            ).map(move |message| {
                match message {
                    SequenceEditorMessage::SelfMessage(content) => Message::SequenceEditorMessage(content),
                    SequenceEditorMessage::SequenceChange(content) => Message::Sequence(content),
                    SequenceEditorMessage::SynthCommand(content) => Message::SynthCommand(content),
                }
            }))
            .push(Row::new()
                .push(Button::new(&mut self.play_button, Text::new("Play"))
                    .on_press(Message::SynthCommand(SynthCommand::Play)))
                .push(Button::new(&mut self.stop_button, Text::new("Stop"))
                    .on_press(Message::SynthCommand(SynthCommand::Stop)))
                .height(Length::Shrink)
            )
            .into()
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
