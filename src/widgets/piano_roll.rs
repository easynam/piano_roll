use iced_native::{Rectangle, Point, Widget, Hasher, Layout, Length, Event, Clipboard, MouseCursor, Background, Color, Vector};
use iced_native::layout::{Node, Limits};
use iced_wgpu::{Renderer, Primitive, Defaults};
use std::{cmp::max, sync::Mutex};
use iced_native::input::{mouse, keyboard, ButtonState};
use crate::widgets::piano_roll::Action::{Dragging, Resizing};
use iced::Element;
use crate::widgets::piano_roll::HoverState::{CanDrag, CanResize, OutOfBounds};
use crate::scroll_zoom::{ScrollZoomState};
use crate::helpers::RectangleHelpers;
use std::ops::{Rem, Mul, Sub, Div};
use crate::sequence::{Note, Sequence, SequenceChange};
use crate::sequence::SequenceChange::{Update, Add, Remove};
use crate::widgets::barlines::{QuantizeGrid, SimpleGrid, LineType};
use iced_native::input::keyboard::ModifiersState;

const DEFAULT_KEY_HEIGHT: f32 = 20.0;
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
}

pub struct PianoRollSettings {
    quantize: Box<QuantizeGrid>,
}

impl Default for PianoRollSettings {
    fn default() -> Self {
        PianoRollSettings {
            quantize: Box::new(SimpleGrid { ticks_per_16th: 32, }),
        }
    }
}

enum HoverState {
    None,
    OutOfBounds,
    CanDrag(usize),
    CanResize(usize),
}

enum Action {
    None,
    Deleting,
    Dragging(Point, usize, Note),
    Resizing(Point, usize, Note),
}

impl Default for PianoRollState {
    fn default() -> Self {
        PianoRollState {
            action: Action::None,
            hover: HoverState::None,
            modifiers: ModifiersState::default(),
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

    fn note_rect(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        Rectangle {
            x: (note.tick as f32 * DEFAULT_TICK_WIDTH - self.scroll_zoom_state.x.scroll()) * self.scroll_zoom_state.x.scale(bounds.width) + bounds.x,
            y: (note.note as f32 * DEFAULT_KEY_HEIGHT - self.scroll_zoom_state.y.scroll()) * self.scroll_zoom_state.y.scale(bounds.height) + bounds.y,
            width: note.length as f32 * self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH,
            height: self.scroll_zoom_state.y.scale(bounds.height) * DEFAULT_KEY_HEIGHT,
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
        _cursor_position: Point,
    ) -> (Primitive, MouseCursor) {
        let bounds = layout.bounds();

        let grid = self.settings.quantize.get_grid_lines((self.scroll_zoom_state.x.view_start / DEFAULT_TICK_WIDTH) as u32, (self.scroll_zoom_state.x.view_end / DEFAULT_TICK_WIDTH) as u32);

        let lines = grid.iter()
            .map(|line| {
                let x = line.tick as f32 * DEFAULT_TICK_WIDTH * self.scroll_zoom_state.x.scale(bounds.width);

                let colour = match line.line_type {
                    LineType::Bar => Color::from_rgb(0.1,0.1,0.1),
                    LineType::Beat => Color::from_rgb(0.1,0.1,0.1),
                    LineType::InBetween => Color::from_rgb(0.2,0.2,0.2),
                };

                let thickness = match line.line_type {
                    LineType::Bar => 2.0,
                    LineType::Beat => 1.0,
                    LineType::InBetween => 1.0,
                };

                Primitive::Quad {
                    bounds: Rectangle {
                        x: (x - thickness/2.0 - self.scroll_zoom_state.x.view_start * self.scroll_zoom_state.x.scale(bounds.width) + bounds.x).round(),
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
            .collect();

        (
            Primitive::Clip {
                bounds: layout.bounds(),
                offset: Vector::default(),
                content: Box::new(Primitive::Group {
                    primitives: vec![
                        Primitive::Quad {
                            bounds,
                            background: Background::Color(Color::from_rgb(0.3,0.3,0.3)),
                            border_radius: 0,
                            border_width: 0,
                            border_color: Color::BLACK,
                        },
                        Primitive::Group {
                            primitives: lines
                        },
                        Primitive::Group {
                            primitives: self.notes.lock().unwrap().iter()
                                .map(|note| {
                                    Primitive::Quad {
                                        bounds: self.note_rect(note, bounds),
                                        background: Background::Color(Color::from_rgb(1.0, 0.8, 0.4)),
                                        border_radius: 0,
                                        border_width: 1,
                                        border_color: Color::BLACK,
                                    }
                                })
                                .collect()
                        }
                    ]
                })
            },
            match self.state.action {
                Dragging(_, _, _) => MouseCursor::Grabbing,
                Resizing(_, _, _) => MouseCursor::ResizingHorizontally,
                Action::None => match self.state.hover {
                    HoverState::None => MouseCursor::Idle,
                    HoverState::OutOfBounds => MouseCursor::OutOfBounds,
                    HoverState::CanDrag(_) => MouseCursor::Grab,
                    HoverState::CanResize(_) => MouseCursor::ResizingHorizontally,
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
        let offset_cursor = cursor_position + Vector {
            x: self.scroll_zoom_state.x.scroll() * self.scroll_zoom_state.x.scale(bounds.width),
            y: self.scroll_zoom_state.y.scroll() * self.scroll_zoom_state.y.scale(bounds.height)
        };
        let notes = self.notes.lock().unwrap();

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    match self.state.action {
                        Dragging(drag_start, note_id, original) => {
                            if let Some(note) = notes.get(note_id) {
                                let offset = Point::new(
                                    offset_cursor.x - drag_start.x,
                                    offset_cursor.y - drag_start.y,
                                );

                                let x_offset = (offset.x / (self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH)).round() as i32;
                                let y_offset = (offset.y / (self.scroll_zoom_state.y.scale(bounds.height) * DEFAULT_KEY_HEIGHT)).round() as i32;

                                let tick = max(0, original.tick as i32 + x_offset) as u32;
                                let tick = self.settings.quantize.quantize_tick(tick);

                                messages.push( (self.on_change)(Update(
                                    note_id,
                                    Note {
                                        tick,
                                        note: max(0, original.note as i32 + y_offset) as u8,
                                        ..*note
                                    }
                                )));
                            }
                        },
                        Resizing(drag_start, note_id, original) => {
                            if let Some(note) = notes.get(note_id) {
                                let x_offset = ((offset_cursor.x - drag_start.x) / (self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH)).round() as i32;

                                messages.push( (self.on_change)(Update(
                                    note_id,
                                    Note {
                                        length: max(1, original.length as i32 + x_offset) as u32,
                                        ..*note
                                    }
                                )));
                            }
                        },
                        Action::Deleting => {
                            self.update_hover(layout, cursor_position, bounds, &notes);
                            self.delete_hovered(messages);
                        },
                        Action::None => {
                            self.update_hover(layout, cursor_position, bounds, &notes);
                        },
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Pressed, } => {
                    match self.state.hover {
                        HoverState::OutOfBounds => {}
                        HoverState::None => {
                            let tick =  ((offset_cursor.x - bounds.x) / (self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH)) as u32;
                            let tick = self.settings.quantize.quantize_tick(tick);

                            let note = Note {
                                tick,
                                note: ((offset_cursor.y - bounds.y) / (self.scroll_zoom_state.y.scale(bounds.height) * DEFAULT_KEY_HEIGHT)) as u8,
                                length: 40,
                            };

                            messages.push( (self.on_change)(Add(note)));
                            self.state.action = Dragging(offset_cursor.clone(), notes.len(), note);
                        },
                        CanDrag(idx) => {
                            self.state.action = Dragging(offset_cursor.clone(), idx, notes[idx].clone());
                        },
                        CanResize(idx) => {
                            self.state.action = Resizing(offset_cursor.clone(), idx, notes[idx].clone());
                        },
                    }
                }
                mouse::Event::Input { button: mouse::Button::Right, state: ButtonState::Pressed, } => {
                    self.delete_hovered(messages)
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
