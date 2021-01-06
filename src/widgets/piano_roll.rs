use iced_native::{Rectangle, Point, Widget, Hasher, Layout, Length, Event, Clipboard, MouseCursor, Background, Color, Vector};
use iced_native::layout::{Node, Limits};
use iced_wgpu::{Renderer, Primitive, Defaults};
use std::{cmp::max, sync::Mutex};
use iced_native::input::{mouse, keyboard, ButtonState};
use crate::widgets::piano_roll::Action::{Dragging, Resizing, Selecting};
use iced::Element;
use crate::widgets::piano_roll::HoverState::{CanDrag, CanResize, OutOfBounds};
use crate::scroll_zoom::{ScrollZoomState};
use crate::helpers::RectangleHelpers;
use crate::sequence::{Note, Sequence, SequenceChange, Pitch};
use crate::sequence::SequenceChange::{Update, Add, Remove};
use crate::widgets::tick_grid::{TickGrid, SimpleGrid, LineType};
use iced_native::input::keyboard::ModifiersState;
use std::cmp::min;
use crate::widgets::pitch_grid::{PitchGrid, TetGrid};
use crate::widgets::pitch_grid;

const DEFAULT_OCTAVE_HEIGHT: f32 = 200.0;
const DEFAULT_TICK_WIDTH: f32 = 1.0;

pub struct PianoRoll<'a, Message> {
    state: &'a mut PianoRollState,
    notes: &'a Mutex<Sequence>,
    on_change: Box<dyn Fn(SequenceChange) -> Message + 'a>,
    scroll_zoom_state: &'a ScrollZoomState,
    settings: &'a PianoRollSettings,
}

pub struct PianoRollState {
    action: Action,
    hover: HoverState,
    modifiers: ModifiersState,
    selection: Vec<usize>,
}

pub struct PianoRollSettings {
    tick_grid: Box<dyn TickGrid>,
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
    CanDrag(usize),
    CanResize(usize),
}

#[derive(PartialEq)]
enum Action {
    None,
    Deleting,
    Dragging(usize, i32),
    Resizing(usize, i32),
    Selecting(i32, Pitch),
}

impl Default for PianoRollState {
    fn default() -> Self {
        PianoRollState {
            action: Action::None,
            hover: HoverState::None,
            modifiers: ModifiersState::default(),
            selection: vec![],
        }
    }
}

impl PianoRollState {
    pub fn new() -> PianoRollState {
        PianoRollState::default()
    }
}

impl<'a, Message> PianoRoll<'a, Message> {
    pub fn new<F>(
        state: &'a mut PianoRollState,
        notes: &'a Mutex<Vec<Note>>,
        on_change: F,
        scroll_zoom_state: &'a ScrollZoomState,
        settings: &'a PianoRollSettings
    ) -> Self
        where
            F: 'a + Fn(SequenceChange) -> Message,
    {
        Self {
            state,
            notes,
            scroll_zoom_state,
            on_change: Box::new(on_change),
            settings
        }
    }

    fn selection_rect(&self, start_tick: i32, start_note: &Pitch, end_tick: i32, end_note: &Pitch, bounds: &Rectangle) -> Rectangle {
        let from_tick = min(start_tick, end_tick);
        let to_tick = max(start_tick, end_tick);
        let from_note = min(start_note, end_note);
        let to_note = max(start_note, end_note);

        let inner = Rectangle {
            x: from_tick as f32 * DEFAULT_TICK_WIDTH,
            y: -to_note.to_f32() * DEFAULT_OCTAVE_HEIGHT - DEFAULT_OCTAVE_HEIGHT/24.0,
            width: (to_tick - from_tick + 1) as f32 * DEFAULT_TICK_WIDTH,
            height: (to_note.clone() - from_note.clone() + Pitch::new(1, 12)).to_f32() * DEFAULT_OCTAVE_HEIGHT,
        };

        self.scroll_zoom_state.inner_rect_to_screen(inner, &bounds)
    }

    fn note_rect(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        let height = self.scroll_zoom_state.y.scale(bounds.height) * DEFAULT_OCTAVE_HEIGHT / 12.0;

        Rectangle {
            x: (note.tick as f32 * DEFAULT_TICK_WIDTH - self.scroll_zoom_state.x.scroll()) * self.scroll_zoom_state.x.scale(bounds.width) + bounds.x,
            y: (-note.pitch.to_f32() * DEFAULT_OCTAVE_HEIGHT - self.scroll_zoom_state.y.scroll()) * self.scroll_zoom_state.y.scale(bounds.height) + bounds.y - height/2.0,
            width: note.length as f32 * self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH,
            height,
        }
    }

    fn note_resize_rect(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        self.note_rect(note, bounds).handle_right()
    }

    fn update_hover(&mut self, layout: Layout, cursor_position: Point, bounds: Rectangle, notes: &Vec<Note>) {
        if layout.bounds().contains(cursor_position) {
            let resize = notes.iter()
                .position(|note| {
                    self.note_resize_rect(note, bounds).contains(cursor_position)
                });

            let hovered = notes.iter()
                .position(|note| {
                    self.note_rect(note, bounds).contains(cursor_position)
                });

            match resize {
                None => {
                    match hovered {
                        None => {
                            self.state.hover = HoverState::None;
                        }
                        Some(idx) => {
                            self.state.hover = CanDrag(idx);
                        }
                    }
                }
                Some(idx) => {
                    match hovered {
                        None => {
                            self.state.hover = CanResize(idx);
                        }
                        Some(hover_idx) => {
                            if idx == hover_idx {
                                self.state.hover = CanResize(idx);
                            } else {
                                self.state.hover = CanDrag(hover_idx);
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
                self.state.action = Action::Deleting;
            },
            CanDrag(idx) => {
                messages.push((self.on_change)(Remove(idx)));
                self.state.action = Action::Deleting;
            },
            CanResize(_) => {
                self.state.action = Action::Deleting;
            },
            _ => {},
        }
    }

    fn draw_tick_grid(&self, bounds: Rectangle) -> Vec<Primitive> {
        let lines = {
            let grid = self.settings.tick_grid.get_grid_lines((self.scroll_zoom_state.x.view_start / DEFAULT_TICK_WIDTH) as i32, (self.scroll_zoom_state.x.view_end / DEFAULT_TICK_WIDTH) as i32);

            grid.iter()
                .map(|line| {
                    let x = line.tick as f32 * DEFAULT_TICK_WIDTH * self.scroll_zoom_state.x.scale(bounds.width);

                    let colour = match line.line_type {
                        LineType::Bar => Color::from_rgb(0.0, 0.0, 0.0),
                        LineType::Beat => Color::from_rgb(0.1, 0.1, 0.1),
                        LineType::InBetween => Color::from_rgb(0.15, 0.15, 0.15),
                    };

                    let thickness = match line.line_type {
                        LineType::Bar => 2.0,
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
                        border_radius: 0,
                        border_width: 0,
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
                Pitch::from_octave_f32(self.scroll_zoom_state.y.view_start / DEFAULT_OCTAVE_HEIGHT),
                Pitch::from_octave_f32(self.scroll_zoom_state.y.view_end / DEFAULT_OCTAVE_HEIGHT)
            );

            grid.iter()
                .map(|line| {
                    let y = line.pitch.to_f32() * DEFAULT_OCTAVE_HEIGHT;

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
                        border_radius: 0,
                        border_width: 0,
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
    ) -> (Primitive, MouseCursor) {
        let bounds = layout.bounds();

        let inner_cursor = self.scroll_zoom_state.screen_to_inner(cursor_position, &bounds);
        let cursor_tick = (inner_cursor.x / DEFAULT_TICK_WIDTH) as i32;
        let cursor_note = Pitch::new(-(12.0 * inner_cursor.y / DEFAULT_OCTAVE_HEIGHT).round() as i32, 12);

        let tick_grid_lines = self.draw_tick_grid(bounds);
        let pitch_grid_lines = self.draw_pitch_grid(bounds);

        let mut layers = vec![
            Primitive::Quad {
                bounds,
                background: Background::Color(Color::from_rgb(0.2,0.2,0.2)),
                border_radius: 0,
                border_width: 0,
                border_color: Color::BLACK,
            },
            Primitive::Group {
                primitives: tick_grid_lines
            },
            Primitive::Group {
                primitives: pitch_grid_lines
            },
            Primitive::Group {
                primitives: self.notes.lock().unwrap().iter().enumerate()
                    .map(|(id, note)| {
                        let colour = match self.state.selection.contains(&id) {
                            true => Color::from_rgb(0.6, 0.9, 1.0),
                            false => Color::from_rgb(1.0, 0.8, 0.4),
                        };

                        Primitive::Quad {
                            bounds: self.note_rect(note, bounds),
                            background: Background::Color(colour),
                            border_radius: 0,
                            border_width: 1,
                            border_color: Color::BLACK,
                        }
                    })
                    .collect()
            },
        ];

        if let Selecting(start_tick, start_note) = &self.state.action {
            layers.push(
                Primitive::Quad {
                    bounds: self.selection_rect(*start_tick, start_note, cursor_tick, &cursor_note, &bounds),
                    background: Background::Color(Color::TRANSPARENT),
                    border_radius: 2,
                    border_width: 2,
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
                Dragging( .. ) => MouseCursor::Grabbing,
                Resizing( .. ) => MouseCursor::ResizingHorizontally,
                Action::None => match self.state.hover {
                    HoverState::None => MouseCursor::Idle,
                    HoverState::OutOfBounds => MouseCursor::OutOfBounds,
                    HoverState::CanDrag( .. ) => MouseCursor::Grab,
                    HoverState::CanResize( .. ) => MouseCursor::ResizingHorizontally,
                },
                _ => MouseCursor::Idle,
            },
        )
    }

    fn hash_layout(&self, _state: &mut Hasher) {
        // use std::hash::Hash;
    }

    fn on_event(&mut self, event: Event, layout: Layout<'_>, cursor_position: Point, messages: &mut Vec<Message>, _renderer: &Renderer, _clipboard: Option<&dyn Clipboard>) {
        let bounds = layout.bounds();

        let inner_cursor = self.scroll_zoom_state.screen_to_inner(cursor_position, &bounds);
        let cursor_tick = (inner_cursor.x / DEFAULT_TICK_WIDTH) as i32;
        let cursor_note = Pitch::new(-(12.0 * inner_cursor.y / DEFAULT_OCTAVE_HEIGHT).round() as i32, 12);

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

                                let mut selected_notes: Vec<(usize, &Note)> = self.state.selection.iter()
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
                                        messages.push( (self.on_change)(Update(note_id, new_note)));
                                    }
                                }
                            }
                        },
                        Resizing(note_id, drag_offset) => {
                            if let Some(note) = notes.get(*note_id) {
                                let length = cursor_tick - note.tick - drag_offset;

                                let mut selected_notes: Vec<(usize, &Note)> = self.state.selection.iter()
                                    .filter_map(|id| notes.get(*id).map(|note| (*id, note)))
                                    .collect();

                                if selected_notes.is_empty() {
                                    selected_notes.push((*note_id, &note))
                                }

                                let min_length = selected_notes.iter().map(|(_, note)| note.length).min().unwrap();

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

                            self.state.selection = notes.iter().enumerate()
                                .filter(|(_id, note)| note.tick <= to_tick && note.end_tick() >= from_tick && note.pitch <= to_note && note.pitch >= from_note)
                                .map(|(id, _note)| id)
                                .collect();
                        }
                        Action::None => { },
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Pressed, } => {
                    if self.state.modifiers.control {
                        self.state.action = Selecting(cursor_tick, cursor_note);
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
                                        self.state.action = Resizing(notes.len(), cursor_tick - tick);
                                        let note = Note { tick, pitch: cursor_note.clone(), length: 0 };
                                        messages.push( (self.on_change)(Add(note)));
                                    }
                                    false => {
                                        self.state.action = Dragging(notes.len(), cursor_tick - tick);
                                        let note = Note { tick, pitch: cursor_note.clone(), length: 32 };
                                        messages.push( (self.on_change)(Add(note)));
                                    }
                                };

                                self.state.selection.clear();
                            },
                            CanDrag(idx) => {
                                let note = &notes[idx];
                                self.state.action = Dragging(idx, cursor_tick - note.tick);
                                if !self.state.selection.contains(&idx) {
                                    self.state.selection.clear();
                                }
                            },
                            CanResize(idx) => {
                                let note = &notes[idx];
                                self.state.action = Resizing(idx, cursor_tick - note.tick - note.length);
                                if !self.state.selection.contains(&idx) {
                                    self.state.selection.clear();
                                }
                            },
                        }
                    } }
                mouse::Event::Input { button: mouse::Button::Right, state: ButtonState::Pressed, } => {
                    //TODO prevent doubling up on deletes
                    self.state.action = Action::Deleting;

                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Released, } => {
                    self.state.action = Action::None;
                }
                mouse::Event::Input { button: mouse::Button::Right, state: ButtonState::Released, } => {
                    self.state.action = Action::None;
                }
                _ => {}
            }
            Event::Keyboard(keyboard::Event::Input { modifiers, .. }) => {
                self.state.modifiers = modifiers;
            }
            _ => {}
        }
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
