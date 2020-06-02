use iced_wgpu::{Primitive, Renderer};
use iced_native::{Element, Layout, MouseCursor, Point, Rectangle};
use crate::stack;

impl stack::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Rectangle, Element<'_, Message, Self>)],
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let mut mouse_cursor = MouseCursor::OutOfBounds;

        (
            Primitive::Group {
                primitives: content
                    .iter()
                    .zip(layout.children())
                    .map(|((_, child), layout)| {
                        let (primitive, new_mouse_cursor) =
                            child.draw(self, defaults, layout, cursor_position);

                        if new_mouse_cursor > mouse_cursor {
                            mouse_cursor = new_mouse_cursor;
                        }

                        primitive
                    })
                    .collect(),
            },
            mouse_cursor,
        )
    }
}