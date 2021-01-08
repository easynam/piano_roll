use iced::Element;
use iced_native::{Background, Hasher, Layout, Length, Point, Rectangle, Vector, Widget};
use iced_native::layout::{Limits, Node};
use iced_native::mouse::Interaction;
use iced_wgpu::{Color, Defaults, Primitive, Renderer};

use crate::scroll_zoom::ScrollScaleAxis;
use crate::widgets::piano_roll::PianoRollSettings;
use crate::widgets::tick_grid::LineType;

pub struct Timeline<'a> {
    scroll: &'a ScrollScaleAxis,
    settings: &'a PianoRollSettings,
}

impl<'a> Timeline<'a> {
    pub fn new(scroll: &'a ScrollScaleAxis, settings: &'a PianoRollSettings) -> Self {
        Self { scroll, settings }
    }
}

impl<'a, Message> Widget<Message, Renderer> for Timeline<'a> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Units(20)
    }

    fn layout(&self, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(limits.height(Length::Units(20)).max())
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, Interaction) {
        // TODO: abstract out
        let bounds = layout.bounds();
        let grid = self.settings.tick_grid.get_grid_lines(self.scroll.view_start as i32, self.scroll.view_end as i32);

        let bar_lines = grid.iter()
            .filter_map(|line| {
                if line.line_type == LineType::Bar {
                    let x = line.tick as f32 * self.scroll.scale(bounds.width);

                    Some(Primitive::Quad {
                        bounds: Rectangle {
                            x: (x - 1.0 - self.scroll.view_start * self.scroll.scale(bounds.width) + bounds.x).round(),
                            y: bounds.y,
                            width: 2.0,
                            height: bounds.height
                        },
                        background: Background::Color(Color::BLACK),
                        border_radius: 0.0,
                        border_width: 0.0,
                        border_color: Color::BLACK
                    })
                } else {
                    None
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
                            background: Background::Color(Color::from_rgb(0.4,0.4,0.4)),
                            border_radius: 0.0,
                            border_width: 0.0,
                            border_color: Color::BLACK,
                        },
                        Primitive::Group {
                            primitives: bar_lines,
                        }
                    ]
                })
            },
            Interaction::Idle
        )
    }

    fn hash_layout(&self, _state: &mut Hasher) {

    }
}

impl<'a, Message> Into<Element<'a, Message>>
for Timeline<'a>
    where
        Message: 'a,
{
    fn into(self) -> Element<'a, Message> {
        Element::new(self)
    }
}
