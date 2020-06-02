//! Distribute content vertically.

use iced_native::{
    layout, Background, Color, Element, Hasher, Layout, Length,
    MouseCursor, Point, Size, Widget, Rectangle, Event, Clipboard,
};
use iced_wgpu::{Defaults, Primitive};
use std::u32;
use std::hash::Hash;

/// A container that distributes its contents vertically.
///
/// A [`Stack`] will try to fill the horizontal space of its container.
///
/// [`Stack`]: struct.Stack.html
#[allow(missing_debug_implementations)]
pub struct Stack<'a, Message, Renderer> {
    padding: u16,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    children: Vec<(Rectangle, Element<'a, Message, Renderer>)>,
}

impl<'a, Message, Renderer> Stack<'a, Message, Renderer> {
    pub fn new() -> Self {
        Stack {
            padding: 0,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            children: Vec::new(),
        }
    }

    pub fn with_children(children: Vec<(Rectangle, Element<'a, Message, Renderer>)>) -> Self {
        Stack {
            padding: 0,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            children,
        }
    }

    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn push<E>(mut self, child: E, rect: Rectangle) -> Self
        where
            E: Into<Element<'a, Message, Renderer>>,
    {
        self.children.push((rect, child.into()));
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
for Stack<'a, Message, Renderer>
    where
        Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        let children = self
            .children
            .iter()
            .filter_map(|(rect, element)| {
                let size = Size::new(rect.width, rect.height);

                let mut node =
                    element.layout(renderer, &layout::Limits::new(size, size));

                node.move_to(Point::new(rect.x, rect.y));

                Some(node)
            })
            .collect();

        layout::Node::with_children(
            limits.resolve(Size::ZERO),
            children
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(defaults, &self.children, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);

        for (_, child) in &self.children {
            child.hash_layout(state);
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        self.children.iter_mut().zip(layout.children()).for_each(
            |((_, child), layout)| {
                child.on_event(
                    event.clone(),
                    layout,
                    cursor_position,
                    messages,
                    renderer,
                    clipboard,
                )
            },
        );
    }
}

/// The renderer of a [`Stack`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Stack`] in your user interface.
///
/// [`Stack`]: struct.Stack.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: iced_native::Renderer + Sized {
    /// Draws a [`Stack`].
    ///
    /// It receives:
    /// - the children of the [`Stack`]
    /// - the [`Layout`] of the [`Stack`] and its children
    /// - the cursor position
    ///
    /// [`Stack`]: struct.Stack.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Rectangle, Element<'_, Message, Self>)],
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Stack<'a, Message, Renderer>>
for Element<'a, Message, Renderer>
    where
        Renderer: 'a + self::Renderer,
        Message: 'a,
{
    fn from(
        stack: Stack<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(stack)
    }
}