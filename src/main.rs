use iced::{Element, Settings, Sandbox, Column};
use iced_native::Container;
use crate::piano_roll::{PianoRoll, Note, SequenceChange};
use std::fmt::Debug;
use crate::scroll_zoom::{ScrollZoomState, ScrollZoomBarX, ScrollScaleAxisChange};

mod piano_roll;
mod scroll_zoom;
mod handles;

pub fn main() {
    App::run(Settings::default())
}

struct App {
    piano_roll_1: piano_roll::State,
    scroll_zoom: ScrollZoomState,
    scroll_bar: scroll_zoom::ScrollZoomBarState,
    notes: Vec<Note>,
}

#[derive(Debug)]
enum Message {
    Sequence(SequenceChange),
    Scroll(ScrollScaleAxisChange),
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App {
            piano_roll_1: piano_roll::State::new(),
            scroll_zoom: Default::default(),
            scroll_bar: scroll_zoom::ScrollZoomBarState::new(),
            notes: vec!(),
        }
    }

    fn title(&self) -> String {
        "wow".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Sequence(change) => match change {
                SequenceChange::Add(note) => {
                    self.notes.push(note);
                },
                SequenceChange::Remove(idx) => {
                    self.notes.remove(idx);
                },
                SequenceChange::Update(idx, note) => {
                    self.notes[idx] = note;
                },
            },
            Message::Scroll(scroll) => match scroll {
                ScrollScaleAxisChange::Left(new_pos) => {
                    self.scroll_zoom.x.view_start = new_pos
                },
                ScrollScaleAxisChange::Right(new_pos) => {
                    self.scroll_zoom.x.view_end = new_pos
                },
                _ => {}
            }
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .push(Container::new(
                PianoRoll::new(&mut self.piano_roll_1, &self.notes, Message::Sequence, &self.scroll_zoom))
                .max_height(600)
            ).padding(10)
            .push(Container::new(
                ScrollZoomBarX::new(
                    &mut self.scroll_bar, &self.scroll_zoom.x, Message::Scroll, true
                )
            ).padding(40))
            // .push(Container::new(PianoRoll::new(&mut self.piano_roll_2, &self.notes, Sequence)).max_height(360))
            .into()
    }
}
