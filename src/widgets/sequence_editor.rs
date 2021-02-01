use crate::widgets::piano_roll::{PianoRoll, PianoRollSettings, PianoRollMessage, PianoRollState};
use crate::widgets::timeline::{Timeline, TimelineState};
use iced::{Element, Column, Row, Space, Length};
use crate::widgets::scroll_bar::{Orientation, ScrollZoomBar, ScrollZoomBarState};
use crate::scroll_zoom::{ScrollZoomState, ScrollScaleAxisChange, ScrollScaleAxis};
use std::sync::{Arc, Mutex};
use crate::sequence::{Sequence, SequenceChange};
use crate::audio::{PlaybackState, SynthCommand};

use SequenceEditorMessage::SelfMessage;
use SequenceEditorSelfMessage::{ScrollUpdateX, ScrollUpdateY};

pub struct SequenceEditor {
    timeline: TimelineState,
    piano_roll: PianoRollState,
    scroll_zoom: ScrollZoomState,
    scroll_bar_x: ScrollZoomBarState,
    scroll_bar_y: ScrollZoomBarState,
}

#[derive(Debug, Clone)]
pub enum SequenceEditorMessage {
    SequenceChange(SequenceChange),
    SynthCommand(SynthCommand),
    SelfMessage(SequenceEditorSelfMessage),
}

#[derive(Debug, Clone)]
pub enum SequenceEditorSelfMessage {
    PianoRoll(PianoRollMessage),
    ScrollUpdateX(ScrollScaleAxisChange),
    ScrollUpdateY(ScrollScaleAxisChange),
}

impl Default for SequenceEditor {
    fn default() -> Self {
        Self {
            timeline: TimelineState::new(),
            piano_roll: Default::default(),
            scroll_zoom: ScrollZoomState {
                x: ScrollScaleAxis::new(0.0,32.0*32.0, 0.0, 32.0*32.0*4.0),
                y: ScrollScaleAxis::new(-1.5, 3.0, -4.0, 8.0),
            },
            scroll_bar_x: Default::default(),
            scroll_bar_y: Default::default()
        }
    }
}

impl SequenceEditor {
    pub fn view<'a>(&'a mut self,
            notes: &'a Arc<Mutex<Sequence>>,
            settings: &'a PianoRollSettings,
            playback_state: &'a PlaybackState,
    ) -> Element<'a, SequenceEditorMessage> {
        Column::new()
            .push(Row::new()
                .push(Timeline::new(
                    &self.scroll_zoom.x,
                    &settings,
                    SequenceEditorMessage::SynthCommand,
                    &mut self.timeline,
                    &playback_state,
                ))
                .push(Space::new(Length::Units(20), Length::Shrink))
                .height(Length::Shrink)
            )
            .push(Row::new()
                .push(PianoRoll::new(
                    &mut self.piano_roll,
                    &notes,
                    SequenceEditorMessage::SequenceChange,
                    |message| SelfMessage(SequenceEditorSelfMessage::PianoRoll(message)),
                    SequenceEditorMessage::SynthCommand,
                    &self.scroll_zoom,
                    &settings,
                    &playback_state,
                ))
                .push(ScrollZoomBar::new(
                    &mut self.scroll_bar_y,
                    &self.scroll_zoom.y,
                    |message| SelfMessage(ScrollUpdateY(message)),
                    Orientation::Vertical,
                    false,
                ))
                .height(Length::Fill)
            )
            .push(Row::new()
                .push(ScrollZoomBar::new(
                    &mut self.scroll_bar_x,
                    &self.scroll_zoom.x,
                    |message| SelfMessage(ScrollUpdateX(message)),
                    Orientation::Horizontal,
                    true,
                ))
                .push(Space::new(Length::Units(20), Length::Shrink))
                .height(Length::Shrink)
            )
            .height(Length::Fill)
            .into()
    }

    pub fn update(&mut self, message: SequenceEditorSelfMessage, notes: &Arc<Mutex<Sequence>>,) {
        match message {
            ScrollUpdateX(scroll) => match scroll {
                ScrollScaleAxisChange::Left(new_pos) => {
                    self.scroll_zoom.x.view_start = new_pos
                },
                ScrollScaleAxisChange::Right(new_pos) => {
                    self.scroll_zoom.x.view_end = new_pos
                },
                _ => {}
            },
            ScrollUpdateY(scroll) => match scroll {
                ScrollScaleAxisChange::Left(new_pos) => {
                    self.scroll_zoom.y.view_start = new_pos
                },
                ScrollScaleAxisChange::Right(new_pos) => {
                    self.scroll_zoom.y.view_end = new_pos
                },
                _ => {}
            },
            SequenceEditorSelfMessage::PianoRoll(action) => {
                self.piano_roll.on_event(action, &*notes.lock().unwrap());
            }
        }
    }
}