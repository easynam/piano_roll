use iced::{Color, Element};

use iced_native::{Widget, Hasher, Layout, Length, Point, MouseCursor, Background, Event, Clipboard, Rectangle, Vector};
use iced_native::layout::{Limits, Node};
use iced_wgpu::{Renderer, Defaults, Primitive};
use iced_native::input::{mouse, ButtonState};
use crate::handles::Handles;

pub struct ScrollScaleAxis {
    pub view_start: f32,
    pub view_end: f32,
    pub content_size: f32,
}

impl ScrollScaleAxis {
    pub fn scroll(&self) -> f32 {
        self.view_start
    }

    pub fn scale(&self, bounds_width: f32) -> f32 {
        bounds_width / (self.view_end - self.view_start)
    }

    pub fn view_width(&self) -> f32 {
        self.view_end - self.view_start
    }

    pub fn view_proportion(&self) -> f32 {
        self.view_width() / self.content_size
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollScaleAxisChange {
    ContentSize(f32),
    Left(f32),
    Right(f32),
}

#[derive(Default)]
pub struct ScrollZoomState {
    pub x: ScrollScaleAxis,
    pub y: ScrollScaleAxis,
}

impl Default for ScrollScaleAxis {
    fn default() -> Self {
        ScrollScaleAxis {
            view_start: 0.0,
            view_end: 1000.0,
            content_size: 2000.0,
        }
    }
}

pub struct ScrollZoomBarState {
    action: Action,
    hover: HoverState,
}

impl Default for ScrollZoomBarState {
    fn default() -> Self {
        ScrollZoomBarState {
            action: Action::None,
            hover: HoverState::None,
        }
    }
}

impl ScrollZoomBarState {
    pub fn new() -> ScrollZoomBarState {
        ScrollZoomBarState::default()
    }
}

enum HoverState {
    None,
    OutOfBounds,
    CanDrag,
    CanResizeRight,
    CanResizeLeft,
}

enum Action {
    None,
    Dragging(f32),
    ResizingRight(f32),
    ResizingLeft(f32),
}

pub struct ScrollZoomBarX<'a, Message> {
    state: &'a mut ScrollZoomBarState,
    axis: &'a ScrollScaleAxis,
    on_change: Box<dyn Fn(ScrollScaleAxisChange) -> Message + 'a>,
    infinite_scroll: bool,
}

const MIN_SCROLLBAR_SIZE: f32 = 8.0;

impl<'a, Message> ScrollZoomBarX<'a, Message> {
    pub fn new<F>(state: &'a mut ScrollZoomBarState, axis: &'a ScrollScaleAxis, on_change: F, infinite_scroll: bool) -> Self
        where
            F: 'a + Fn(ScrollScaleAxisChange) -> Message,
    {
        ScrollZoomBarX {
            state,
            axis,
            on_change: Box::new(on_change),
            infinite_scroll,
        }
    }

    fn bar_offset(&self, bounds: &Rectangle) -> f32 {
        bounds.x + (self.axis.view_start / self.axis.content_size) * bounds.width
    }

    fn bar_width(&self, bounds: &Rectangle) -> f32 {
        self.axis.view_proportion() * bounds.width
    }

    fn bar_rect(&self, bounds: &Rectangle) -> Rectangle {
        let mut x = self.bar_offset(bounds);
        let mut width = self.bar_width(bounds);

        if x > bounds.x + bounds.width - width {
            x = x.min(bounds.x + bounds.width - MIN_SCROLLBAR_SIZE);
            width = bounds.x + bounds.width - x;
        }

        Rectangle {
            width,
            height: bounds.height,
            x,
            y: bounds.y,
        }
    }
}

impl<'a, Message> Widget<Message, Renderer> for ScrollZoomBarX<'a, Message> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Units(1)
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

        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Quad {
                        bounds,
                        background: Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
                        border_radius: 0,
                        border_width: 0,
                        border_color: Color::BLACK,
                    },
                    Primitive::Quad {
                        bounds: self.bar_rect(&bounds),
                        background: Background::Color(Color::from_rgb(0.8, 0.8, 0.8)),
                        border_radius: 0,
                        border_width: 1,
                        border_color: Color::BLACK,
                    },
                ]
            },
            match self.state.action {
                Action::Dragging( .. ) => MouseCursor::Grabbing,
                Action::ResizingRight( .. ) => MouseCursor::ResizingHorizontally,
                Action::ResizingLeft( .. ) => MouseCursor::ResizingHorizontally,
                Action::None => match self.state.hover {
                    HoverState::None => MouseCursor::Idle,
                    HoverState::OutOfBounds => MouseCursor::OutOfBounds,
                    HoverState::CanDrag => MouseCursor::Grab,
                    HoverState::CanResizeRight => MouseCursor::ResizingHorizontally,
                    HoverState::CanResizeLeft => MouseCursor::ResizingHorizontally,
                }
            },
        )
    }

    fn hash_layout(&self, _state: &mut Hasher) {
        // use std::hash::Hash;
    }

    fn on_event(&mut self, _event: Event, layout: Layout<'_>, cursor_position: Point, messages: &mut Vec<Message>, _renderer: &Renderer, _clipboard: Option<&dyn Clipboard>) {
        let bounds = layout.bounds();
        let offset_cursor = cursor_position - Vector {
            x: bounds.x,
            y: bounds.y,
        };

        match _event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    match self.state.action {
                        Action::None => {
                            match layout.bounds().contains(cursor_position) {
                                true => {
                                    let bar = self.bar_rect(&layout.bounds());
                                    if bar.handle_right().contains(cursor_position) {
                                        self.state.hover = HoverState::CanResizeRight
                                    }
                                    else if bar.handle_left().contains(cursor_position) {
                                        self.state.hover = HoverState::CanResizeLeft
                                    }
                                    else if bar.contains(cursor_position) {
                                        self.state.hover = HoverState::CanDrag
                                    }
                                    else {
                                        self.state.hover = HoverState::None
                                    }
                                }
                                false => self.state.hover = HoverState::OutOfBounds
                            }
                        }
                        Action::Dragging(offset) => {
                            let mut start = offset_cursor.x - offset - bounds.x;
                            let mut end = start + self.bar_width(&bounds);

                            dbg!(offset_cursor.x, start, end);

                            if start < 0.0 {
                                end -= start;
                                start = 0.0;
                            }
                            if !self.infinite_scroll && end > bounds.width {
                                start -= end - bounds.width;
                                end = bounds.width;
                            }

                            messages.push((self.on_change)(ScrollScaleAxisChange::Left(start * (self.axis.content_size / bounds.width))));
                            messages.push((self.on_change)(ScrollScaleAxisChange::Right(end * (self.axis.content_size / bounds.width))));

                        }
                        Action::ResizingLeft(offset) => {
                            let mut start = offset_cursor.x - offset - bounds.x;
                            if start < 0.0 {
                                start = 0.0;
                            }
                            messages.push((self.on_change)(ScrollScaleAxisChange::Left(start * self.axis.content_size / bounds.width)));
                        }
                        Action::ResizingRight(offset) => {
                            let mut end = offset_cursor.x - offset - bounds.x;
                            if !self.infinite_scroll && end > bounds.width {
                                end = bounds.width;
                            }
                            messages.push((self.on_change)(ScrollScaleAxisChange::Right(end * self.axis.content_size / bounds.width)));
                        }
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Pressed, } => {
                    match self.state.hover {
                        HoverState::CanDrag => self.state.action = Action::Dragging(offset_cursor.x - self.bar_offset(&bounds)),
                        HoverState::CanResizeRight => self.state.action = Action::ResizingRight(offset_cursor.x - self.bar_offset(&bounds) - self.bar_width(&bounds)),
                        HoverState::CanResizeLeft => self.state.action = Action::ResizingLeft(offset_cursor.x - self.bar_offset(&bounds)),
                        _ => {}
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Released, } => {
                    self.state.action = Action::None;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl<'a, Message> Into<Element<'a, Message>>
for ScrollZoomBarX<'a, Message>
    where
        Message: 'a,
{
    fn into(self) -> Element<'a, Message> {
        Element::new(self)
    }
}