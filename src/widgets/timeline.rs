use iced::Element;
use iced_native::{Background, Hasher, Layout, Length, Point, Rectangle, Vector, Widget, HorizontalAlignment, VerticalAlignment, Event, Clipboard, mouse, keyboard, Size};
use iced_native::layout::{Limits, Node};
use iced_native::mouse::Interaction;
use iced_wgpu::{Color, Defaults, Primitive, Renderer};

use crate::scroll_zoom::ScrollScaleAxis;
use crate::widgets::piano_roll::PianoRollSettings;
use crate::widgets::tick_grid::LineType;
use crate::audio::{Command, PlaybackState};
use iced_native::event::Status;
use iced_native::keyboard::Modifiers;
use std::cmp::max;
use iced_wgpu::triangle::Mesh2D;
use iced_graphics::widget::canvas::{Frame, Path, Fill, Stroke};

pub struct Timeline<'a, Message> {
    scroll: &'a ScrollScaleAxis,
    settings: &'a PianoRollSettings,
    on_synth_command: Box<dyn Fn(Command) -> Message + 'a>,
    state: &'a mut TimelineState,
    playback_state: &'a PlaybackState,
}

pub struct TimelineState {
    action: Action,
    modifiers: Modifiers,
}

impl TimelineState {
    pub fn new() -> Self {
        Self {
            action: Action::None,
            modifiers: Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    Seeking,
}

impl<'a, Message> Timeline<'a, Message> {
    pub fn new<FS>(
        scroll: &'a ScrollScaleAxis,
        settings: &'a PianoRollSettings,
        on_synth_command: FS,
        state: &'a mut TimelineState,
        playback_state: &'a PlaybackState,
    ) -> Self
        where
            FS: 'a + Fn(Command) -> Message,
    {
        Self { scroll, settings, on_synth_command: Box::new(on_synth_command), state, playback_state }
    }

    fn seek(&mut self, cursor_position: Point, messages: &mut Vec<Message>, bounds: Rectangle) {
        let mut cursor_tick = self.scroll.screen_to_inner(cursor_position.x, bounds.x, bounds.width) as i32;

        if !self.state.modifiers.alt {
            cursor_tick = self.settings.tick_grid.quantize_tick(cursor_tick);
        }
        cursor_tick = max(0, cursor_tick);

        messages.push((self.on_synth_command)(Command::Seek(cursor_tick)));
    }
}

impl<'a, Message> Widget<Message, Renderer> for Timeline<'a, Message> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Units(30)
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

        let playback_cursor_x = self.playback_state.playback_cursor as f32 * self.scroll.scale(bounds.width) - 15.0;

        let mut frame = Frame::new(Size::new(30.0,30.0));
        let path = Path::new(|path| {
            path.move_to(Point::new(15.0,28.0));
            path.line_to(Point::new(2.0,15.0));
            path.line_to(Point::new(28.0,15.0));
            path.close();
        });
        frame.fill(&path, Color::from_rgb(0.8, 0.8, 0.8));
        frame.stroke(&path, Stroke::default().with_color(Color::from_rgb(0.0, 0.0, 0.0)));

        let cursor = Primitive::Translate {
            translation: Vector::new(playback_cursor_x, bounds.y),
            content: Box::new(frame.into_geometry().into_primitive()),
        };

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
                        },
                        cursor
                    ]
                })
            },
            Interaction::Idle
        )
    }

    fn hash_layout(&self, _state: &mut Hasher) {

    }

    fn on_event(&mut self, event: Event, layout: Layout<'_>, cursor_position: Point, messages: &mut Vec<Message>, _renderer: &Renderer, _clipboard: Option<&dyn Clipboard>) -> Status {
        match event {
            Event::Mouse(event) => match event {
                mouse::Event::CursorMoved { .. } => {
                    match self.state.action {
                        Action::None => {
                            Status::Ignored
                        }
                        Action::Seeking => {
                            let bounds = layout.bounds();
                            self.seek(cursor_position, messages, bounds);

                            Status::Captured
                        }
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let bounds = layout.bounds();
                    if bounds.contains(cursor_position) {
                        self.state.action = Action::Seeking;
                        self.seek(cursor_position, messages, bounds);

                        Status::Captured
                    } else {
                        Status::Ignored
                    }
                },
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.state.action = Action::None;
                    Status::Captured
                },
                _ => Status::Ignored,
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.state.modifiers = modifiers;
                Status::Ignored
            },
            _ => Status::Ignored,
        }
    }
}


impl<'a, Message> Into<Element<'a, Message>>
for Timeline<'a, Message>
    where
        Message: 'a,
{
    fn into(self) -> Element<'a, Message> {
        Element::new(self)
    }
}
