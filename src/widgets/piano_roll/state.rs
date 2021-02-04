use iced_native::keyboard::Modifiers;
use crate::sequence::{NoteId, Pitch, Sequence, Note, SequenceChange};
use crate::widgets::piano_roll::state::Action::{Dragging, Resizing};
use derive_more::{Constructor};
use std::cmp::{max, min};
use crate::widgets::piano_roll::{PianoRollMessage, PianoRollSettings};
use crate::audio::SynthCommand;

pub struct PianoRollState {
    pub(crate) action: Action,
    pub(crate) hover: HoverState,
    pub(crate) modifiers: Modifiers,
    pub(crate) selection: Vec<NoteId>,
    pub(crate) cursor: Cursor,
}

pub enum HoverState {
    None,
    OutOfBounds,
    CanDrag(NoteId),
    CanResize(NoteId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    Deleting,
    Dragging(NoteId, i32),
    Resizing(NoteId, i32),
    Selecting(i32, Pitch),
}

#[derive(Debug, Clone)]
pub enum PianoRollSelfMessage {
    Action(Action),
    DragLastCreatedNote(i32),
    ResizeLastCreatedNote(i32),
}

impl Default for PianoRollState {
    fn default() -> Self {
        PianoRollState {
            action: Action::None,
            hover: HoverState::None,
            modifiers: Modifiers::default(),
            selection: vec![],
            cursor: Cursor::default(),
        }
    }
}

#[derive(Constructor, Default, PartialEq)]
pub struct Cursor {
    pub tick: i32,
    pub pitch: Pitch,
}

impl PianoRollState {
    pub fn on_event(&mut self, event: PianoRollSelfMessage, notes: &Sequence) {
        match event {
            PianoRollSelfMessage::Action(action) => self.action = action,
            PianoRollSelfMessage::DragLastCreatedNote(cursor_tick) => {
                if let Some((id, note)) = notes.last_added() {
                    self.action = Dragging(id, cursor_tick - note.tick);
                }
            }
            PianoRollSelfMessage::ResizeLastCreatedNote(cursor_tick) => {
                if let Some((id, note)) = notes.last_added() {
                    self.action = Resizing(id, cursor_tick - note.tick);
                }
            }
        }
    }

    pub(crate) fn delete_hovered(&mut self, messages: &mut Vec<PianoRollMessage>) {
        match self.hover {
            HoverState::None => {
                messages.push(PianoRollMessage::SelfMessage(PianoRollSelfMessage::Action(Action::Deleting)));
            },
            HoverState::CanDrag(id) => {
                messages.push(PianoRollMessage::SequenceChange(SequenceChange::Remove(id)));
                messages.push(PianoRollMessage::SelfMessage(PianoRollSelfMessage::Action(Action::Deleting)));
            },
            HoverState::CanResize(_) => {
                messages.push(PianoRollMessage::SelfMessage(PianoRollSelfMessage::Action(Action::Deleting)));
            },
            _ => {},
        }
    }

    pub fn update_cursor(&mut self, cursor: Cursor, messages: &mut Vec<PianoRollMessage>, notes: &Sequence, settings: &PianoRollSettings) {
        if self.cursor != cursor {
            match &self.action {
                Action::Dragging(note_id, drag_offset) => {
                    if let Some(note) = notes.get(*note_id) {
                        let quantize_offset = note.tick - settings.tick_grid.quantize_tick(note.tick);
                        let mut tick = max(0, cursor.tick - drag_offset);
                        if !self.modifiers.alt {
                            tick = settings.tick_grid.quantize_tick(tick - quantize_offset) + quantize_offset;
                        }

                        let mut selected_notes: Vec<(NoteId, &Note)> = self.selection.iter()
                            .filter_map(|id| notes.get(*id).map(|note| (*id, note)))
                            .collect();

                        if selected_notes.is_empty() {
                            selected_notes.push((*note_id, &note))
                        }

                        let min_tick = selected_notes.iter().map(|(_, note)| note.tick).min().unwrap();

                        let min_note = selected_notes.iter().map(|(_, note)| note.pitch.clone()).min().unwrap();
                        let max_note = selected_notes.iter().map(|(_, note)| note.pitch.clone()).max().unwrap();

                        let tick_offset = max(-min_tick, tick - note.tick);
                        // arbitrary max note
                        let note_offset = (cursor.pitch.clone() - note.pitch.clone()).clamp(Pitch::new(-4, 1) - min_note, Pitch::new(4, 1) - max_note);

                        // todo: optional mode for irregular grids?
                        for (note_id, note) in selected_notes {
                            let new_note = Note {
                                tick: note.tick + tick_offset,
                                pitch: note.pitch.clone() + note_offset.clone(),
                                ..*note
                            };

                            if note != &new_note {
                                messages.push(PianoRollMessage::SequenceChange(SequenceChange::Update(note_id, new_note.clone())));
                            }
                            if &note.pitch != &new_note.pitch {
                                messages.push(PianoRollMessage::SynthCommand(SynthCommand::StartPreview(new_note.pitch.clone())));
                            }
                        }
                    }
                },
                Action::Resizing(note_id, drag_offset) => {
                    if let Some(note) = notes.get(*note_id) {
                        let quantize_offset = note.tick + note.length - settings.tick_grid.quantize_tick(note.tick + note.length);
                        let mut tick = cursor.tick - drag_offset;
                        if !self.modifiers.alt {
                            tick = settings.tick_grid.quantize_tick(tick - quantize_offset) + quantize_offset;
                        }
                        let length = tick - note.tick;

                        let mut selected_notes: Vec<(NoteId, &Note)> = self.selection.iter()
                            .filter_map(|id| notes.get(*id).map(|note| (*id, note)))
                            .collect();

                        if selected_notes.is_empty() {
                            selected_notes.push((*note_id, &note))
                        }

                        let mut min_length = selected_notes.iter().map(|(_, note)| note.length).min().unwrap();
                        if !self.modifiers.alt {
                            min_length -= settings.tick_grid.grid_size(tick);
                        }

                        let length_offset =  max(-min_length, length - note.length);

                        for (note_id, note) in selected_notes {
                            let new_note = Note {
                                length: note.length + length_offset,
                                ..note.clone()
                            };

                            if note != &new_note {
                                messages.push( PianoRollMessage::SequenceChange(SequenceChange::Update(note_id, new_note)));
                            }
                        }
                    }
                },
                Action::Deleting => {
                    self.delete_hovered(messages);
                },
                Action::Selecting(start_tick, start_note) => {
                    let from_tick = min(start_tick, &cursor.tick).clone();
                    let to_tick = max(start_tick, &cursor.tick).clone();
                    let from_note = min(start_note, &cursor.pitch).clone();
                    let to_note = max(start_note, &cursor.pitch).clone();

                    self.selection = notes.iter()
                        .filter(|(_id, note)| note.tick <= to_tick && note.end_tick() >= from_tick && note.pitch <= to_note && note.pitch >= from_note)
                        .map(|(id, _note)| id)
                        .collect();
                }
                Action::None => { },
            }

            self.cursor = cursor;
        }

    }
}
