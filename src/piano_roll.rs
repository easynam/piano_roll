use iced_native::{Rectangle, Point, Widget, Command, Hasher, Layout, Length, Event, Clipboard, MouseCursor, Background, Color, Size, Vector};
use iced_native::layout::{Node, Limits};
use iced_wgpu::{Renderer, Primitive, Defaults};
use std::cmp::max;
use iced_native::input::{mouse, ButtonState};
use crate::piano_roll::Action::{Dragging, Resizing};
use iced::Element;
use crate::piano_roll::HoverState::{CanDrag, CanResize, OutOfBounds};
use crate::piano_roll::SequenceChange::{Add, Update, Remove};
use crate::scroll_zoom::{ScrollZoomState};
use iced_native::widget::svg::Handle;
use crate::handles::Handles;

const SELECT_MIN_WIDTH: f32 = 12.0;
const RESIZE_LEFT: f32 = 8.0;
const RESIZE_RIGHT: f32 = 8.0;
const DEFAULT_KEY_HEIGHT: f32 = 20.0;
const DEFAULT_TICK_WIDTH: f32 = 1.0;

pub struct PianoRoll<'a, Message> {
    state: &'a mut State,
    notes: &'a Vec<Note>,
    on_change: Box<dyn Fn(SequenceChange) -> Message + 'a>,
    scroll_zoom_state: &'a ScrollZoomState,
}

pub struct State {
    action: Action,
    hover: HoverState,
}

enum HoverState {
    None,
    OutOfBounds,
    CanDrag(usize),
    CanResize(usize),
}

enum Action {
    None,
    Dragging(Point, usize, Note),
    Resizing(Point, usize, Note),
}

impl Default for State {
    fn default() -> Self {
        State {
            action: Action::None,
            hover: HoverState::None,
        }
    }
}

impl State {
    pub fn new() -> State {
        State::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    tick: u32,
    note: u8,
    length: u32,
}

#[derive(Debug, Clone)]
pub enum SequenceChange {
    Add(Note),
    Remove(usize),
    Update(usize, Note),
}

impl<'a, Message> PianoRoll<'a, Message> {
    pub fn new<F>(state: &'a mut State, notes: &'a Vec<Note>, on_change: F, scroll_zoom_state: &'a ScrollZoomState) -> Self
        where
            F: 'a + Fn(SequenceChange) -> Message,
    {
        Self {
            state,
            notes,
            scroll_zoom_state,
            on_change: Box::new(on_change),
        }
    }

    fn note_rect(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        Rectangle {
            x: note.tick as f32 * self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH + bounds.x,
            y: note.note as f32 * self.scroll_zoom_state.y.scale(bounds.width) * DEFAULT_KEY_HEIGHT + bounds.y,
            width: note.length as f32 * self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH,
            height: self.scroll_zoom_state.y.scale(bounds.width) * DEFAULT_KEY_HEIGHT,
        }
    }

    fn note_resize_rect(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        self.note_rect(note, bounds).handle_right()
    }

    fn note_resize_rect_l(&self, note: &Note, bounds: Rectangle,) -> Rectangle {
        self.note_rect(note, bounds).handle_left()
    }
}

impl<'a, Message> Widget<Message, Renderer> for PianoRoll<'a, Message> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
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

        (
            Primitive::Clip {
                bounds: layout.bounds(),
                offset: Vector::default(),
                content: Box::new(Primitive::Group {
                    primitives: self.notes.iter()
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
                }
            },
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
    }

    fn on_event(&mut self, _event: Event, layout: Layout<'_>, cursor_position: Point, messages: &mut Vec<Message>, _renderer: &Renderer, _clipboard: Option<&dyn Clipboard>) {
        let bounds = layout.bounds();
        let offset_cursor = cursor_position; //later will use scroll offset maybe? maybe scroll outside of this widget tho lol
        // let corner = Point::new(layout.bounds().x, layout.bounds().y);

        match _event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { x, y } => {
                    match self.state.action {
                        Dragging(drag_start, note_id, original) => {
                            if let Some(note) = self.notes.get(note_id) {
                                let offset = Point::new(
                                    offset_cursor.x - drag_start.x,
                                    offset_cursor.y - drag_start.y,
                                );

                                let x_offset = (offset.x / (self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH)).round() as i32;
                                let y_offset = (offset.y / (self.scroll_zoom_state.y.scale(bounds.width) * DEFAULT_KEY_HEIGHT)).round() as i32;

                                messages.push( (self.on_change)(Update(
                                    note_id,
                                    Note {
                                        tick: max(0, original.tick as i32 + x_offset) as u32,
                                        note: max(0, original.note as i32 + y_offset) as u8,
                                        ..*note
                                    }
                                )));
                            }
                        },
                        Resizing(drag_start, note_id, original) => {
                            if let Some(note) = self.notes.get(note_id) {
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
                        Action::None => {
                            if layout.bounds().contains(cursor_position) {
                                let resize = self.notes.iter()
                                    .position(|note| {
                                        self.note_resize_rect(note, bounds).contains(offset_cursor)
                                    });

                                let hovered = self.notes.iter()
                                    .position(|note| {
                                        self.note_rect(note, bounds).contains(offset_cursor)
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
                        },
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Pressed, } => {
                    match self.state.hover {
                        HoverState::OutOfBounds => {}
                        HoverState::None => {
                            let note = Note {
                                tick: ((offset_cursor.x - bounds.x) / (self.scroll_zoom_state.x.scale(bounds.width) * DEFAULT_TICK_WIDTH)) as u32,
                                note: ((offset_cursor.y - bounds.y) / (self.scroll_zoom_state.y.scale(bounds.width) * DEFAULT_KEY_HEIGHT)) as u8,
                                length: 40,
                            };

                            messages.push( (self.on_change)(Add(note)));
                            self.state.action = Dragging(offset_cursor.clone(), self.notes.len(), note);
                        },
                        CanDrag(idx) => {
                            self.state.action = Dragging(offset_cursor.clone(), idx, self.notes[idx].clone());
                        },
                        CanResize(idx) => {
                            self.state.action = Resizing(offset_cursor.clone(), idx, self.notes[idx].clone());
                        },
                    }
                }
                mouse::Event::Input { button: mouse::Button::Right, state: ButtonState::Pressed, } => {
                    match self.state.hover {
                        HoverState::OutOfBounds => {},
                        HoverState::None => {},
                        CanDrag(idx) => {
                            messages.push( (self.on_change)(Remove(idx)));
                        },
                        CanResize(idx) => {
                            messages.push( (self.on_change)(Remove(idx)));
                        },
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Released, } => {
                    self.state.action = Action::None;
                }
                _ => {}
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