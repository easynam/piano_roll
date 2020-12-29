use iced::{Element, Settings, Sandbox, Column};
use iced_native::Container;
use widgets::piano_roll::{PianoRoll, PianoRollSettings};
use std::fmt::Debug;
use crate::scroll_zoom::{ScrollZoomState, ScrollScaleAxisChange};
use crate::sequence::{SequenceChange, Sequence, update_sequence};
use crate::widgets::scroll_bar::{ScrollZoomBarState, ScrollZoomBarX};
use widgets::piano_roll;

mod sequence;
mod widgets;
mod scroll_zoom;
mod handles;

pub fn main() {
    App::run(Settings::default())
}

struct App {
    piano_roll_1: piano_roll::PianoRollState,
    scroll_zoom: ScrollZoomState,
    scroll_bar: ScrollZoomBarState,
    notes: Sequence,
    settings: PianoRollSettings,
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
            piano_roll_1: piano_roll::PianoRollState::new(),
            scroll_zoom: Default::default(),
            scroll_bar: ScrollZoomBarState::new(),
            notes: vec!(),
            settings: PianoRollSettings::default(),
        }
    }

    fn title(&self) -> String {
        "wow".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Sequence(change) => update_sequence(&mut self.notes, change),
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
                PianoRoll::new(&mut self.piano_roll_1, &self.notes, Message::Sequence, &self.scroll_zoom, &self.settings))
                .max_height(600)
            )
            .push(Container::new(
                ScrollZoomBarX::new(
                    &mut self.scroll_bar, &self.scroll_zoom.x, Message::Scroll, true
                )).max_height(20)
            )
            .padding(20)
            // .push(Container::new(PianoRoll::new(&mut self.piano_roll_2, &self.notes, Sequence)).max_height(360))
            .into()
    }
}
