use iced::Element;
use iced_native::{Background, Hasher, Layout, Length, Point, Rectangle, Vector, Widget, HorizontalAlignment, VerticalAlignment};
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
        Node::new(limits.height(Length::Units(30)).max())
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
        let grid = self.settings.tick_grid.get_grid_lines(self.scroll.view_start as i32 - 40, self.scroll.view_end as i32);

        let bar_lines = grid.iter()
            .map(|line| {
                let colour = match line.line_type {
                    LineType::Bar(_) => Color::from_rgb(0.0, 0.0, 0.0),
                    LineType::Beat => Color::from_rgb(0.1, 0.1, 0.1),
                    LineType::InBetween => Color::from_rgb(0.25, 0.25, 0.25),
                };

                let thickness = match line.line_type {
                    LineType::Bar(_) => 2.0,
                    _ => 1.0,
                };

                let height = match line.line_type {
                    LineType::Bar(_) => bounds.height,
                    _ => 8.0,
                };

                let x = line.tick as f32 * self.scroll.scale(bounds.width);

                Primitive::Quad {
                    bounds: Rectangle {
                        x: (x - thickness / 2.0 - self.scroll.view_start * self.scroll.scale(bounds.width) + bounds.x).round(),
                        y: bounds.y,
                        width: thickness,
                        height: height
                    },
                    background: Background::Color(colour),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::BLACK
                }
            })
            .collect();

        let numbers = grid.iter()
            .filter_map(|line| {
                if let LineType::Bar(bar_number) = line.line_type {
                    let x = line.tick as f32 * self.scroll.scale(bounds.width);

                    Some(Primitive::Text {
                        content: bar_number.to_string(),
                        bounds: Rectangle {
                            x: (x - self.scroll.view_start * self.scroll.scale(bounds.width) + bounds.x).round() + 6.0,
                            y: bounds.y + bounds.height/2.0 + 6.0,
                            width: 40.0,
                            height: bounds.height - 6.0
                        },
                        color: Color::WHITE,
                        size: 17.0,
                        font: Default::default(),
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Center
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
                        },
                        Primitive::Group {
                            primitives: numbers,
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