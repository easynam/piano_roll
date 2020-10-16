use iced::{Vector, Color, Element};

use crate::{handles, Message};
use iced_native::{Widget, Hasher, Layout, Length, Point, MouseCursor, Background, Event, Clipboard, Rectangle};
use iced_native::layout::{Limits, Node};
use iced_wgpu::{Renderer, Defaults, Primitive};
use iced_native::input::{mouse, ButtonState};
use crate::handles::Handles;

pub struct ScrollZoomControls {

}

pub struct ScrollScaleAxis {
    pub scroll: f32,
    pub scale: f32,
    pub content_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollScaleAxisChange {
    Scroll(f32),
    Scale(f32),
    ContentSize(f32),
}

#[derive(Default)]
pub struct ScrollZoomState {
    pub x: ScrollScaleAxis,
    pub y: ScrollScaleAxis,
}

impl Default for ScrollScaleAxis {
    fn default() -> Self {
        ScrollScaleAxis {
            scroll: 0.0,
            scale: 1.0,
            content_size: 10000.0,
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
    Dragging(Point, f32),
    ResizingRight(Point, f32),
    ResizingLeft(Point, f32),
}

pub struct ScrollZoomBarX<'a, Message> {
    state: &'a mut ScrollZoomBarState,
    axis: &'a ScrollScaleAxis,
    on_change: Box<dyn Fn(ScrollScaleAxisChange) -> Message + 'a>,
}

impl<'a, Message> ScrollZoomBarX<'a, Message> {
    pub fn new<F>(state: &'a mut ScrollZoomBarState, axis: &'a ScrollScaleAxis, on_change: F) -> Self
        where
            F: 'a + Fn(ScrollScaleAxisChange) -> Message,
    {
        ScrollZoomBarX {
            state,
            axis,
            on_change: Box::new(on_change)
        }
    }

    fn bar_width(&self, bounds: &Rectangle ) -> f32 {
        bounds.x + self.axis.scroll * self.axis.scale
    }
    
    fn bar_rect(&self, bounds: &Rectangle) -> Rectangle {
        let effective_content_size = f32::max(bounds.width, self.axis.content_size * self.axis.scale);

        let width = bounds.width * (bounds.width / effective_content_size);

        Rectangle {
            width,
            height: bounds.height,
            x: self.bar_width(bounds),
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
        let corner = Point::new(layout.bounds().x, layout.bounds().y);

        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Quad {
                        bounds: layout.bounds(),
                        background: Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
                        border_radius: 0,
                        border_width: 0,
                        border_color: Color::BLACK,
                    },
                    Primitive::Quad {
                        bounds: self.bar_rect(&layout.bounds()),
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

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
    }

    fn on_event(&mut self, _event: Event, layout: Layout<'_>, cursor_position: Point, messages: &mut Vec<Message>, _renderer: &Renderer, _clipboard: Option<&dyn Clipboard>) {

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
                        Action::Dragging(from, old_scroll) => {
                            messages.push((self.on_change)(ScrollScaleAxisChange::Scroll(old_scroll + (cursor_position.x - from.x) / self.axis.scale)))
                        }
                        Action::ResizingRight(from, old_scale) => {
                            let offset = (cursor_position.x - from.x);
                            let bar_width = self.bar_width(&layout.bounds());
                            messages.push((self.on_change)(ScrollScaleAxisChange::Scale(old_scale * (bar_width / (offset + bar_width)))))
                        }
                        _ => {}
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Pressed, } => {
                    match self.state.hover {
                        HoverState::CanDrag => self.state.action = Action::Dragging(cursor_position, self.axis.scroll),
                        HoverState::CanResizeRight => self.state.action = Action::ResizingRight(cursor_position, self.axis.scale),
                        _ => {}
                    }
                }
                mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Released, } => {
                    self.state.action = Action::None;
                }
                mouse::Event::WheelScrolled { delta } => {}
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