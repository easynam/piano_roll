// For now, to implement a custom native widget you will need to add
// `iced_native` and `iced_wgpu` to your dependencies.
//
// Then, you simply need to define your widget type and implement the
// `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
//
// Of course, you can choose to make the implementation renderer-agnostic,
// if you wish to, by creating your own `Renderer` trait, which could be
// implemented by `iced_wgpu` and other renderers.
use iced_native::{layout, Background, Color, Element, Hasher, Layout, Length, MouseCursor, Point, Size, Widget, Event, Clipboard};
use iced_wgpu::{Defaults, Primitive, Renderer};
use iced_native::input::{mouse, ButtonState};

// #[derive(Debug, Clone, Copy)]
// pub enum DragEvent {
//     Moved(Point),
//     Dropped(Point),
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum DragStatus {
//     None,
//     Dragging(Point),
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
// pub struct State {
//     drag_status: DragStatus,
// }

#[allow(missing_debug_implementations)]
pub struct NoteWidget {
    // on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    // state: State,
}

impl NoteWidget {
    pub fn new() -> Self {
        Self {  }
    }

    // pub fn on_drag<F>(mut self, f: F) -> Self
    //     where
    //         F: 'a + Fn(DragEvent) -> Message,
    // {
    //     self.on_drag = Some(Box::new(f));
    //     self
    // }

}

impl<Message> Widget<Message, Renderer> for NoteWidget {
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(_limits.max())
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> (Primitive, MouseCursor) {
        (
            Primitive::Quad {
                bounds: layout.bounds(),
                background: Background::Color(Color::from_rgb(1.0, 0.8, 0.4)),
                border_radius: 4,
                border_width: 1,
                border_color: Color::BLACK,
            },
            MouseCursor::OutOfBounds,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
    }

    // fn on_event(
    //     &mut self,
    //     event: Event,
    //     layout: Layout<'_>,
    //     cursor_position: Point,
    //     messages: &mut Vec<Message>,
    //     renderer: &Renderer,
    //     clipboard: Option<&dyn Clipboard>,
    // ) {
    //     match event {
    //         Event::Mouse(mouse::Event::Input { button: mouse::Button::Left, state }) => {
    //             let bounds = layout.bounds();
    //
    //             match state {
    //                 ButtonState::Pressed => {
    //                     if bounds.contains(cursor_position) {
    //                         DragStatus::Dragging(cursor_position.clone())
    //                     }
    //                 },
    //                 ButtonState::Released => {
    //                     self.state.drag_status = DragStatus::None;
    //                 },
    //             }
    //         }
    //         Event::Mouse(mouse::Event::CursorMoved { x: 0.0, y: 0.0 }) => {
    //             let message = self.on_drag(DragEvent::Moved(cursor_position));
    //             messages.push(message)
    //         }
    //         _ => {}
    //     }
    // }
}

impl<'a, Message> Into<Element<'a, Message, Renderer>>
for NoteWidget
    where
        Message: 'a,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}