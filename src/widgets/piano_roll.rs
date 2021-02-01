use std::{cmp::max, sync::Mutex};
use std::cmp::min;

use iced::Element;
use iced_native::{Background, Clipboard, Color, Event, Hasher, keyboard, Layout, Length, mouse, Point, Rectangle, Vector, Widget};
use iced_native::event::Status;
use iced_native::keyboard::Modifiers;
use iced_native::layout::{Limits, Node};
use iced_native::mouse::Interaction;
use iced_wgpu::{Defaults, Primitive, Renderer};

use crate::audio::{SynthCommand, PlaybackState};
use crate::helpers::RectangleHelpers;
use crate::scroll_zoom::ScrollZoomState;
use crate::sequence::{Note, NoteId, Pitch, Sequence, SequenceChange};
use crate::sequence::SequenceChange::{Add, Remove, Update};
use crate::widgets::piano_roll::Action::{Dragging, Resizing, Selecting};
use crate::widgets::piano_roll::HoverState::{CanDrag, CanResize, OutOfBounds};
use crate::widgets::pitch_grid::{PitchGrid, TetGrid};
use crate::widgets::pitch_grid;
use crate::widgets::tick_grid::{LineType, SimpleGrid, TickGrid};

pub struct PianoRoll<'a, Message> {
    state: &'a mut PianoRollState,
    notes: &'a Mutex<Sequence>,
    on_change: Box<dyn Fn(SequenceChange) -> Message + 'a>,
    on_self_change: Box<dyn Fn(PianoRollMessage) -> Message + 'a>,
    on_synth_command: Box<dyn Fn(SynthCommand) -> Message + 'a>,
    scroll_zoom_state: &'a ScrollZoomState,
    settings: &'a PianoRollSettings,
    playback_state: &'a PlaybackState,
}

pub struct PianoRollState {
    action: Action,
    hover: HoverState,
    modifiers: Modifiers,
    selection: Vec<NoteId>,
}

pub struct PianoRollSettings {
    pub(crate) tick_grid: Box<dyn TickGrid>,
    pitch_grid: Box<dyn PitchGrid>,
}

impl Default for PianoRollSettings {
    fn default() -> Self {
        PianoRollSettings {
            tick_grid: Box::new(SimpleGrid { ticks_per_16th: 32, }),
            pitch_grid: Box::new(TetGrid { tones_per_octave: 12, pattern: vec![
                pitch_grid::LineType::White,
                pitch_grid::LineType::Black,
                pitch_grid::LineType::White,
                pitch_grid::LineType::Black,
                pitch_grid::LineType::White,
                pitch_grid::LineType::White,
                pitch_grid::LineType::Black,
                pitch_grid::LineType::White,
                pitch_grid::LineType::Black,
                pitch_grid::LineType::Tonic,
                pitch_grid::LineType::White,
                pitch_grid::LineType::Black,
            ]}),
        }
    }
}

enum HoverState {
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

impl Default for PianoRollState {
    fn default() -> Self {
        PianoRollState {
            action: Action::None,
            hover: HoverState::None,
            modifiers: Modifiers::default(),
            selection: vec![],
        }
    }
}

impl PianoRollState {
    pub fn on_event(&mut self, event: PianoRollMessage, notes: &Sequence) {
        match event {
            PianoRollMessage::Action(action) => self.action = action,
            PianoRollMessage::DragLastCreatedNote(cursor_tick) => {
                if let Some((id, note)) = notes.last_added() {
                    self.action = Dragging(id, cursor_tick - note.tick);
                }
            }
            PianoRollMessage::ResizeLastCreatedNote(cursor_tick) => {
                if let Some((id, note)) = notes.last_added() {
                    self.action = Resizing(id, cursor_tick - note.tick);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PianoRollMessage {
    Action(Action),
    DragLastCreatedNote(i32),
    ResizeLastCreatedNote(i32),
}

impl<'a, Message> PianoRoll<'a, Message> {
    pub fn new<F, FA, FS>(
        state: &'a mut PianoRollState,
        notes: &'a Mutex<Sequence>,
        on_change: F,
        on_self_change: FA,
        on_synth_command: FS,
        scroll_zoom_state: &'a ScrollZoomState,
        settings: &'a PianoRollSettings,
        playback_state: &'a PlaybackState,
    ) -> Self
        where
            F: 'a + Fn(SequenceChange) -> Message,
            FA: 'a + Fn(PianoRollMessage) -> Message,
            FS: 'a + Fn(SynthCommand) -> Message,
    {
        Self {
            state,
            notes,
            scroll_zoom_state,
            on_change: Box::new(on_change),
            on_self_change: Box::new(on_self_change),
            on_synth_command: Box::new(on_synth_command),
            settings,
            playback_state,
        }
    }

    fn selection_rect(&self, start_tick: i32, start_note: &Pitch, end_tick: i32, end_note: &Pitch, bounds: &Rectangle) -> Rectangle {
        let from_tick = min(start_tick, end_tick);
        let to_tick = max(start_tick, end_tick);
        let from_note = min(start_note, end_note);
        let to_note = max(start_note, end_note);

        let inner = Rectangle {
            x: from_tick as f32,
            y: -to_note.to_f32() - 1.0/24.0,
            width: (to_tick - from_tick + 1) as f32,
            height: (to_note.clone() - from_note.clone() + Pitch::new(1, 12)).to_f32(),
        };

        self.scroll_zoom_state.inner_rect_to_screen(inner, &bounds)
    }

    fn note_rect(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        let height = self.scroll_zoom_state.y.scale(bounds.height) / 12.0;

        Rectangle {
            x: (note.tick as f32 - self.scroll_zoom_state.x.scroll()) * self.scroll_zoom_state.x.scale(bounds.width) + bounds.x,
            y: (-note.pitch.to_f32() - self.scroll_zoom_state.y.scroll()) * self.scroll_zoom_state.y.scale(bounds.height) + bounds.y - height/2.0,
            width: note.length as f32 * self.scroll_zoom_state.x.scale(bounds.width),
            height,
        }
    }

    fn note_resize_rect(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        self.note_rect(note, bounds).handle_right()
    }

    fn update_hover(&mut self, layout: Layout, cursor_position: Point, bounds: Rectangle, notes: &Sequence) {
        if layout.bounds().contains(cursor_position) {
            let resize = notes.iter()
                .find(|(_id, note)| {
                    self.note_resize_rect(note, bounds).contains(cursor_position)
                });

            let hovered = notes.iter()
                .find(|(_id, note)| {
                    self.note_rect(note, bounds).contains(cursor_position)
                });

            match resize {
                None => {
                    match hovered {
                        None => {
                            self.state.hover = HoverState::None;
                        }
                        Some((id, _)) => {
                            self.state.hover = CanDrag(id);
                        }
                    }
                }
                Some((id, _)) => {
                    match hovered {
                        None => {
                            self.state.hover = CanResize(id);
                        }
                        Some((hover_id, _)) => {
                            if id == hover_id {
                                self.state.hover = CanResize(id);
                            } else {
                                self.state.hover = CanDrag(hover_id);
                            }
                        }
                    }
                }
            }
        } else {
            self.state.hover = OutOfBounds
        }
    }

    fn delete_hovered(&mut self, messages: &mut Vec<Message>) {
        match self.state.hover {
            HoverState::None => {
                messages.push((self.on_self_change)(PianoRollMessage::Action(Action::Deleting)));
            },
            CanDrag(id) => {
                messages.push((self.on_change)(Remove(id)));
                messages.push((self.on_self_change)(PianoRollMessage::Action(Action::Deleting)));
            },
            CanResize(_) => {
                messages.push((self.on_self_change)(PianoRollMessage::Action(Action::Deleting)));
            },
            _ => {},
        }
    }

    fn draw_cursor(&self, bounds: Rectangle) -> Primitive {
        let x = self.playback_state.playback_cursor as f32 * self.scroll_zoom_state.x.scale(bounds.width);
        Primitive::Quad {
            bounds: Rectangle {
                x: (x - self.scroll_zoom_state.x.view_start * self.scroll_zoom_state.x.scale(bounds.width) + bounds.x).round(),
                y: bounds.y,
                width: 1.0,
                height: bounds.height
            },
            background: Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::BLACK
        }
    }

    fn draw_tick_grid(&self, bounds: Rectangle) -> Vec<Primitive> {
        let lines = {
            let grid = self.settings.tick_grid.get_grid_lines(self.scroll_zoom_state.x.view_start as i32, self.scroll_zoom_state.x.view_end as i32);

            grid.iter()
                .map(|line| {
                    let x = line.tick as f32 * self.scroll_zoom_state.x.scale(bounds.width);

                    let colour = match line.line_type {
                        LineType::Bar(_) => Color::from_rgb(0.0, 0.0, 0.0),
                        LineType::Beat => Color::from_rgb(0.1, 0.1, 0.1),
                        LineType::InBetween => Color::from_rgb(0.15, 0.15, 0.15),
                    };

                    let thickness = match line.line_type {
                        LineType::Bar(_) => 2.0,
                        LineType::Beat => 1.0,
                        LineType::InBetween => 1.0,
                    };

                    Primitive::Quad {
                        bounds: Rectangle {
                            x: (x - thickness / 2.0 - self.scroll_zoom_state.x.view_start * self.scroll_zoom_state.x.scale(bounds.width) + bounds.x).round(),
                            y: bounds.y,
                            width: thickness,
                            height: bounds.height
                        },
                        background: Background::Color(colour),
                        border_radius: 0.0,
                        border_width: 0.0,
                        border_color: Color::BLACK
                    }
                })
                .collect()
        };
        lines
    }

    fn draw_pitch_grid(&self, bounds: Rectangle) -> Vec<Primitive> {
        let lines = {
            let grid = self.settings.pitch_grid.get_grid_lines(
                Pitch::from_octave_f32(self.scroll_zoom_state.y.view_start),
                Pitch::from_octave_f32(self.scroll_zoom_state.y.view_end)
            );

            grid.iter()
                .map(|line| {
                    let y = line.pitch.to_f32();

                    let colour = match line.line_type {
                        pitch_grid::LineType::Tonic => Color::from([0.5, 1.0, 0.5, 0.3]),
                        pitch_grid::LineType::White => Color::from([1.0, 1.0, 1.0, 0.15]),
                        pitch_grid::LineType::Black => Color::from([1.0, 1.0, 1.0, 0.05]),
                    };

                    let thickness = match line.line_type {
                        pitch_grid::LineType::Tonic => 2.0,
                        pitch_grid::LineType::White => 2.0,
                        pitch_grid::LineType::Black => 1.0,
                    };

                    Primitive::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: (self.scroll_zoom_state.y.inner_to_screen(y, bounds.y, bounds.height) - thickness/2.0).round(),
                            width: bounds.width,
                            height: thickness
                        },
                        background: Background::Color(colour),
                        border_radius: 0.0,
                        border_width: 0.0,
                        border_color: Color::BLACK
                    }
                })
                .collect()
        };
        lines
    }
}

impl<'a, Message> Widget<Message, Renderer> for PianoRoll<'a, Message> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(&self, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(limits.max())
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _defaults: &Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, Interaction) {
        let bounds = layout.bounds();

        let inner_cursor = self.scroll_zoom_state.screen_to_inner(cursor_position, &bounds);
        let cursor_tick = inner_cursor.x as i32;
        let cursor_note = Pitch::new(-(12.0 * inner_cursor.y).round() as i32, 12);

        let cursor_lines = self.draw_cursor(bounds);
        let tick_grid_lines = self.draw_tick_grid(bounds);
        let pitch_grid_lines = self.draw_pitch_grid(bounds);

        let mut layers = vec![
            Primitive::Quad {
                bounds,
                background: Background::Color(Color::from_rgb(0.2,0.2,0.2)),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::BLACK,
            },
            Primitive::Group {
                primitives: tick_grid_lines
            },
            Primitive::Group {
                primitives: pitch_grid_lines
            },
            Primitive::Group {
                primitives: self.notes.lock().unwrap().iter()
                    .map(|(id, note)| {
                        let colour = match self.state.selection.contains(&id) {
                            true => Color::from_rgb(0.6, 0.9, 1.0),
                            false => Color::from_rgb(1.0, 0.8, 0.4),
                        };

                        Primitive::Quad {
                            bounds: self.note_rect(note, bounds),
                            background: Background::Color(colour),
                            border_radius: 0.0,
                            border_width: 1.0,
                            border_color: Color::BLACK,
                        }
                    })
                    .collect()
            },
            cursor_lines,
        ];

        if let Selecting(start_tick, start_note) = &self.state.action {
            layers.push(
                Primitive::Quad {
                    bounds: self.selection_rect(*start_tick, start_note, cursor_tick, &cursor_note, &bounds),
                    background: Background::Color(Color::TRANSPARENT),
                    border_radius: 2.0,
                    border_width: 2.0,
                    border_color: Color::WHITE,
                }
            )
        }

        (
            Primitive::Clip {
                bounds: layout.bounds(),
                offset: Vector::default(),
                content: Box::new(Primitive::Group {
                    primitives: layers
                })
            },
            match self.state.action {
                Dragging( .. ) => Interaction::Grabbing,
                Resizing( .. ) => Interaction::ResizingHorizontally,
                Action::None => match self.state.hover {
                    HoverState::None => Interaction::Idle,
                    HoverState::OutOfBounds => Interaction::default(),
                    HoverState::CanDrag( .. ) => Interaction::Grab,
                    HoverState::CanResize( .. ) => Interaction::ResizingHorizontally,
                },
                _ => Interaction::Idle,
            },
        )
    }

    fn hash_layout(&self, _state: &mut Hasher) {
        // use std::hash::Hash;
    }

    fn on_event(&mut self, event: Event, layout: Layout<'_>, cursor_position: Point, messages: &mut Vec<Message>, _renderer: &Renderer, _clipboard: Option<&dyn Clipboard>) -> Status {
        let bounds = layout.bounds();

        let inner_cursor = self.scroll_zoom_state.screen_to_inner(cursor_position, &bounds);
        let cursor_tick = inner_cursor.x as i32;
        let cursor_note = Pitch::new(-(12.0 * inner_cursor.y).round() as i32, 12);

        let notes = self.notes.lock().unwrap();

        self.update_hover(layout, cursor_position, bounds, &notes);

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    match &self.state.action {
                        Dragging(note_id, drag_offset) => {
                            if let Some(note) = notes.get(*note_id) {
                                let quantize_offset = note.tick - self.settings.tick_grid.quantize_tick(note.tick);
                                let mut tick = max(0, cursor_tick - drag_offset);
                                if !self.state.modifiers.alt {
                                    tick = self.settings.tick_grid.quantize_tick(tick - quantize_offset) + quantize_offset;
                                }

                                let mut selected_notes: Vec<(NoteId, &Note)> = self.state.selection.iter()
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
                                let note_offset = (cursor_note - note.pitch.clone()).clamp(Pitch::new(-4, 1) - min_note, Pitch::new(4, 1) - max_note);

                                // todo: optional mode for irregular grids?
                                for (note_id, note) in selected_notes {
                                    let new_note = Note {
                                        tick: note.tick + tick_offset,
                                        pitch: note.pitch.clone() + note_offset.clone(),
                                        ..*note
                                    };

                                    if note != &new_note {
                                        messages.push( (self.on_change)(Update(note_id, new_note.clone())));
                                    }
                                    if &note.pitch != &new_note.pitch {
                                        messages.push((self.on_synth_command)(SynthCommand::StartPreview(new_note.pitch.clone())));
                                    }
                                }
                            }
                        },
                        Resizing(note_id, drag_offset) => {
                            if let Some(note) = notes.get(*note_id) {
                                let quantize_offset = note.tick + note.length - self.settings.tick_grid.quantize_tick(note.tick + note.length);
                                let mut tick = cursor_tick - drag_offset;
                                if !self.state.modifiers.alt {
                                    tick = self.settings.tick_grid.quantize_tick(tick - quantize_offset) + quantize_offset;
                                }
                                let length = tick - note.tick;

                                let mut selected_notes: Vec<(NoteId, &Note)> = self.state.selection.iter()
                                    .filter_map(|id| notes.get(*id).map(|note| (*id, note)))
                                    .collect();

                                if selected_notes.is_empty() {
                                    selected_notes.push((*note_id, &note))
                                }

                                let mut min_length = selected_notes.iter().map(|(_, note)| note.length).min().unwrap();
                                if !self.state.modifiers.alt {
                                    min_length -= self.settings.tick_grid.grid_size(tick);
                                }

                                let length_offset =  max(-min_length, length - note.length);

                                for (note_id, note) in selected_notes {
                                    let new_note = Note {
                                        length: note.length + length_offset,
                                        ..note.clone()
                                    };

                                    if note != &new_note {
                                        messages.push( (self.on_change)(Update(note_id, new_note)));
                                    }
                                }
                            }
                        },
                        Action::Deleting => {
                            self.delete_hovered(messages);
                        },
                        Selecting(start_tick, start_note) => {
                            let from_tick = min(start_tick, &cursor_tick).clone();
                            let to_tick = max(start_tick, &cursor_tick).clone();
                            let from_note = min(start_note, &cursor_note).clone();
                            let to_note = max(start_note, &cursor_note).clone();

                            self.state.selection = notes.iter()
                                .filter(|(_id, note)| note.tick <= to_tick && note.end_tick() >= from_tick && note.pitch <= to_note && note.pitch >= from_note)
                                .map(|(id, _note)| id)
                                .collect();
                        }
                        Action::None => { },
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if self.state.modifiers.control {
                        match self.state.hover {
                            HoverState::OutOfBounds => {}
                            _ => messages.push((self.on_self_change)(PianoRollMessage::Action(Selecting(cursor_tick, cursor_note))))
                        }
                    } else {
                        match self.state.hover {
                            HoverState::OutOfBounds => {}
                            HoverState::None => {
                                let mut tick = cursor_tick;

                                if !self.state.modifiers.alt {
                                    tick = self.settings.tick_grid.quantize_tick(tick);
                                }

                                match self.state.modifiers.shift {
                                    true => {
                                        let length = match self.state.modifiers.alt {
                                            true => 0,
                                            false => self.settings.tick_grid.grid_size(tick),
                                        };
                                        let note = Note { tick, pitch: cursor_note.clone(), length };
                                        messages.push((self.on_synth_command)(SynthCommand::StartPreview(note.pitch.clone())));
                                        messages.push( (self.on_change)(Add(note)));
                                        messages.push((self.on_self_change)(PianoRollMessage::ResizeLastCreatedNote(cursor_tick)));
                                    }
                                    false => { 
                                        let note = Note { tick, pitch: cursor_note.clone(), length: 32 };
                                        messages.push((self.on_synth_command)(SynthCommand::StartPreview(note.pitch.clone())));
                                        messages.push( (self.on_change)(Add(note)));
                                        messages.push((self.on_self_change)(PianoRollMessage::DragLastCreatedNote(cursor_tick)));
                                    }
                                };

                                self.state.selection.clear();
                            },
                            CanDrag(id) => {
                                if let Some(note) = &notes.get(id) {
                                    messages.push((self.on_self_change)(PianoRollMessage::Action(Dragging(id, cursor_tick - note.tick))));
                                    messages.push((self.on_synth_command)(SynthCommand::StartPreview(note.pitch.clone())));
                                    if !self.state.selection.contains(&id) {
                                        self.state.selection.clear();
                                    }
                                    if self.state.modifiers.shift {
                                        match self.state.selection.is_empty() {
                                            true => {
                                                messages.push((self.on_change)(Add((*note).clone())))
                                            },
                                            false => {
                                                for id in &self.state.selection {
                                                    notes.get(*id).map(|note| {
                                                        messages.push((self.on_change)(Add(note.clone())))
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            CanResize(id) => {
                                if let Some(note) = &notes.get(id) {
                                    messages.push((self.on_self_change)(PianoRollMessage::Action(Resizing(id, cursor_tick - note.tick - note.length))));
                                    if !self.state.selection.contains(&id) {
                                        self.state.selection.clear();
                                    }
                                }
                            },
                        }
                    } }
                mouse::Event::ButtonPressed(mouse::Button::Right) => {
                    self.delete_hovered(messages);
                }
                mouse::Event::ButtonReleased( .. ) => {
                    messages.push((self.on_self_change)(PianoRollMessage::Action(Action::None)));
                    messages.push((self.on_synth_command)(SynthCommand::StopPreview));
                }
                _ => {}
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.state.modifiers = modifiers;
            }
            _ => {}
        }

        Status::Captured
    }
}


impl<'a, Message> Into<Element<'a, Message>>
for PianoRoll<'a, Message>
    where
        Message: 'a,
{
    fn into(self) -> Element<'a, Message> {
        Element::new(self)
    }
}
