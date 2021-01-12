use crate::scroll_zoom::{ScrollScaleAxis, ScrollScaleAxisChange};
use iced_native::{Widget, Hasher, Layout, Length, Point, Background, Event, Clipboard, Rectangle, Color, mouse};
use iced_native::event::Status;
use iced_native::layout::{Limits, Node};
use iced_wgpu::{Renderer, Defaults, Primitive};
use crate::helpers::{PointHelpers, RectangleHelpers};
use iced::Element;
use iced_native::mouse::Interaction;

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

pub enum Orientation {
    Horizontal, Vertical
}

pub struct ScrollZoomBar<'a, Message> {
    state: &'a mut ScrollZoomBarState,
    axis: &'a ScrollScaleAxis,
    on_change: Box<dyn Fn(ScrollScaleAxisChange) -> Message + 'a>,
    orientation: Orientation,
    infinite_scroll: bool,
}

const MIN_SCROLLBAR_SIZE: f32 = 8.0;

impl<'a, Message> ScrollZoomBar<'a, Message> {
    pub fn new<F>(state: &'a mut ScrollZoomBarState, axis: &'a ScrollScaleAxis, on_change: F, orientation: Orientation, infinite_scroll: bool) -> Self
        where
            F: 'a + Fn(ScrollScaleAxisChange) -> Message,
    {
        ScrollZoomBar {
            state,
            axis,
            on_change: Box::new(on_change),
            orientation,
            infinite_scroll,
        }
    }

    fn bar_offset(&self, bounds_offset: f32, bounds_size: f32) -> f32 {
        bounds_offset + ((self.axis.view_start - self.axis.content_start) / self.axis.content_size) * bounds_size
    }

    fn bar_position(&self, bounds_offset: f32, bounds_size: f32) -> (f32, f32) {
        let mut offset = self.bar_offset(bounds_offset, bounds_size);
        let mut size = self.axis.view_proportion() * bounds_size;

        if offset > bounds_offset + bounds_size - size {
            offset = offset.min(bounds_offset + bounds_size - MIN_SCROLLBAR_SIZE);
            size = bounds_offset + bounds_size - offset;
        }

        (offset, size)
    }

    fn bar_rect_horizontal(&self, bounds: &Rectangle) -> Rectangle {
        let (x, width) = self.bar_position(bounds.x, bounds.width);

        Rectangle {
            x,
            y: bounds.y,
            width,
            height: bounds.height,
        }
    }

    fn bar_rect_vertical(&self, bounds: &Rectangle) -> Rectangle {
        let (y, height) = self.bar_position(bounds.y, bounds.height);

        Rectangle {
            x: bounds.x,
            y,
            width: bounds.width,
            height,
        }
    }

    fn bar_rect(&self, bounds: &Rectangle) -> Rectangle {
        match self.orientation {
            Orientation::Horizontal => self.bar_rect_horizontal(&bounds),
            Orientation::Vertical => self.bar_rect_vertical(&bounds),
        }
    }

    fn handle_start(&self, rect: Rectangle) -> Rectangle {
        match self.orientation {
            Orientation::Horizontal => rect.handle_left(),
            Orientation::Vertical => rect.handle_up(),
        }
    }

    fn handle_end(&self, rect: Rectangle) -> Rectangle {
        match self.orientation {
            Orientation::Horizontal => rect.handle_right(),
            Orientation::Vertical => rect.handle_down(),
        }
    }

    fn resize_cursor(&self) -> Interaction {
        match self.orientation {
            Orientation::Horizontal => Interaction::ResizingHorizontally,
            Orientation::Vertical => Interaction::ResizingVertically,
        }
    }
}

impl<'a, Message> Widget<Message, Renderer> for ScrollZoomBar<'a, Message> {
    fn width(&self) -> Length {
        match self.orientation {
            Orientation::Horizontal => Length::Fill,
            Orientation::Vertical => Length::Units(20),
        }
    }

    fn height(&self) -> Length {
        match self.orientation {
            Orientation::Horizontal => Length::Units(20),
            Orientation::Vertical => Length::Fill,
        }
    }

    fn layout(&self, _renderer: &Renderer, limits: &Limits) -> Node {
        match self.orientation {
            Orientation::Horizontal => Node::new(limits.height(Length::Units(20)).max()),
            Orientation::Vertical => Node::new(limits.width(Length::Units(20)).max()),
        }
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, Interaction) {
        let bounds = layout.bounds();

        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Quad {
                        bounds,
                        background: Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
                        border_radius: 0.0,
                        border_width: 0.0,
                        border_color: Color::BLACK,
                    },
                    Primitive::Quad {
                        bounds: self.bar_rect(&bounds),
                        background: Background::Color(Color::from_rgb(0.8, 0.8, 0.8)),
                        border_radius: 0.0,
                        border_width: 1.0,
                        border_color: Color::BLACK,
                    },
                ]
            },
            match self.state.action {
                Action::Dragging( .. ) => Interaction::Grabbing,
                Action::ResizingRight( .. ) => self.resize_cursor(),
                Action::ResizingLeft( .. ) => self.resize_cursor(),
                Action::None => match self.state.hover {
                    HoverState::None => Interaction::Idle,
                    HoverState::OutOfBounds => Interaction::default(),
                    HoverState::CanDrag => Interaction::Grab,
                    HoverState::CanResizeRight => self.resize_cursor(),
                    HoverState::CanResizeLeft => self.resize_cursor(),
                }
            },
        )
    }

    fn hash_layout(&self, _state: &mut Hasher) {
        // use std::hash::Hash;
    }

    fn on_event(&mut self, event: Event, layout: Layout<'_>, cursor_position: Point, messages: &mut Vec<Message>, _renderer: &Renderer, _clipboard: Option<&dyn Clipboard>) -> Status {
        let bounds = layout.bounds();
        let cursor_offset = match self.orientation {
            Orientation::Horizontal => cursor_position.normalize_within_bounds(&bounds).x,
            Orientation::Vertical => cursor_position.normalize_within_bounds(&bounds).y,
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    match self.state.action {
                        Action::None => {
                            match layout.bounds().contains(cursor_position) {
                                true => {
                                    let bar = self.bar_rect(&bounds);
                                    if self.handle_end(bar).contains(cursor_position) {
                                        self.state.hover = HoverState::CanResizeRight
                                    }
                                    else if self.handle_start(bar).contains(cursor_position) {
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
                            let mut start = cursor_offset - offset;
                            let mut end = start + self.axis.view_proportion();

                            if start < 0.0 {
                                end -= start;
                                start = 0.0;
                            }
                            if !self.infinite_scroll && end > 1.0 {
                                start -= end - 1.0;
                                end = 1.0;
                            }

                            messages.push((self.on_change)(ScrollScaleAxisChange::Left(start * self.axis.content_size + self.axis.content_start)));
                            messages.push((self.on_change)(ScrollScaleAxisChange::Right(end * self.axis.content_size + self.axis.content_start)));

                        }
                        Action::ResizingLeft(offset) => {
                            let mut start = cursor_offset - offset;
                            if start < 0.0 {
                                start = 0.0;
                            }
                            messages.push((self.on_change)(ScrollScaleAxisChange::Left(start * self.axis.content_size + self.axis.content_start)));
                        }
                        Action::ResizingRight(offset) => {
                            let mut end = cursor_offset - offset;
                            if !self.infinite_scroll && end > 1.0 {
                                end = 1.0;
                            }
                            messages.push((self.on_change)(ScrollScaleAxisChange::Right(end * self.axis.content_size + self.axis.content_start)));
                        }
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    match self.state.hover {
                        HoverState::CanDrag => self.state.action = Action::Dragging(cursor_offset - self.axis.start_proportion()),
                        HoverState::CanResizeRight => self.state.action = Action::ResizingRight(cursor_offset - self.axis.start_proportion() - self.axis.view_proportion()),
                        HoverState::CanResizeLeft => self.state.action = Action::ResizingLeft(cursor_offset - self.axis.start_proportion()),
                        _ => {}
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.state.action = Action::None;
                }
                _ => {}
            },
            _ => {}
        };

        Status::Captured
    }
}

impl<'a, Message> Into<Element<'a, Message>>
for ScrollZoomBar<'a, Message>
    where
        Message: 'a,
{
    fn into(self) -> Element<'a, Message> {
        Element::new(self)
    }
}