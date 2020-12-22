use iced::{Element, Settings, Sandbox, Column, Text, executor};
use crate::stack::Stack;
use iced_native::{Rectangle, Subscription, Event, Command, Point, Button, button, Length, Scrollable};
use crate::note::{NoteWidget};
use iced_native::input::{mouse, ButtonState};
use iced_native::Container;
use std::cmp::{min, max};
use std::ops::Rem;
use crate::piano_roll::{PianoRoll, State, Note, SequenceChange};
use std::fmt::Debug;
use iced_native::scrollable;
use crate::scroll_zoom::{ScrollZoomState, ScrollScaleAxis, ScrollZoomBarX, ScrollScaleAxisChange};

mod stack;
mod stack_renderer;
mod note;
mod piano_roll;
mod scroll_zoom;
mod handles;

pub fn main() {
    App::run(Settings::default())
}

struct App {
    scrollable_1: scrollable::State,
    piano_roll_1: piano_roll::State,
    scroll_zoom: ScrollZoomState,
    piano_roll_2: piano_roll::State,
    scroll_bar: scroll_zoom::ScrollZoomBarState,
    notes: Vec<Note>,
}

#[derive(Debug)]
enum Message {
    Sequence(SequenceChange),
    Scroll(ScrollScaleAxisChange),
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App {
            scrollable_1: Default::default(),
            piano_roll_1: piano_roll::State::new(),
            scroll_zoom: Default::default(),
            piano_roll_2: piano_roll::State::new(),
            scroll_bar: scroll_zoom::ScrollZoomBarState::new(),
            notes: vec!(),
        }
    }

    fn title(&self) -> String {
        "wow".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Sequence(change) => match change {
                SequenceChange::Add(note) => {
                    self.notes.push(note);
                },
                SequenceChange::Remove(idx) => {
                    self.notes.remove(idx);
                },
                SequenceChange::Update(idx, note) => {
                    self.notes[idx] = note;
                },
            },
            Message::Scroll(scroll) => match scroll {
                ScrollScaleAxisChange::Left(new_pos) => {
                    self.scroll_zoom.x.view_start = new_pos
                },
                ScrollScaleAxisChange::Right(new_pos) => {
                    self.scroll_zoom.x.view_end = new_pos
                },
                _ => {}
            }
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .push(Container::new(
                PianoRoll::new(&mut self.piano_roll_1, &self.notes, Message::Sequence, &self.scroll_zoom))
                .max_height(700)
            )
            .push(Container::new(
                ScrollZoomBarX::new(
                    &mut self.scroll_bar, &self.scroll_zoom.x, Message::Scroll
                )
            ))
            // .push(Container::new(PianoRoll::new(&mut self.piano_roll_2, &self.notes, Sequence)).max_height(360))
            .into()
    }
}

// struct PianoRoll {
//     vertical_scale: f32,
//     horizontal_scale: f32,
//     notes: Vec<Note>,
//     cursor: Point,
//     dragging: Option<(Point, usize, Note)>,
//     button: button::State,
// }
//
// impl Default for PianoRoll {
//     fn default() -> Self {
//         PianoRoll {
//             vertical_scale: 20.0,
//             horizontal_scale: 20.0,
//             notes: vec!(
//             ),
//             cursor: Point::new(0.0, 0.0),
//             dragging: None,
//             button: Default::default()
//         }
//     }
// }
//
//
// #[derive(Debug, Clone, Copy)]
// struct Note {
//     tick: u32,
//     note: u8,
//     length: u32,
// }
//
// #[derive(Debug, Clone)]
// enum Message {
//     Event(iced_native::Event),
//     Noop,
// }
//
// impl PianoRoll {
//     fn note_rect(&self, note: &Note) -> Rectangle {
//         Rectangle {
//             x: note.tick as f32 * self.horizontal_scale,
//             y: note.note as f32 * self.vertical_scale,
//             width: note.length as f32 * self.horizontal_scale,
//             height: self.vertical_scale,
//         }
//     }
//
//     fn overlaps(&self, mouse: Point, note: &Note) -> bool {
//         self.note_rect(note).contains(mouse)
//     }
// }
//
// impl Application for PianoRoll {
//     type Executor = executor::Default;
//     type Message = Message;
//     type Flags = ();
//
//     fn new(_flags: ()) -> (Self, Command<Message>) {
//         (PianoRoll::default(), Command::none())
//     }
//
//     fn title(&self) -> String {
//         String::from("hey")
//     }
//
//     fn update(&mut self, message: Message) -> Command<Message> {
//         match message {
//             Message::Event(event) => match event {
//                 Event::Mouse(mouse_event) => match mouse_event {
//                     mouse::Event::CursorMoved { x, y } => {
//                         self.cursor = Point { x, y };
//
//                         if let Some((drag_start, note_id, original)) = self.dragging {
//                             if let Some(note) = self.notes.get_mut(note_id) {
//                                 let offset = Point::new(
//                                     self.cursor.x - drag_start.x,
//                                     self.cursor.y - drag_start.y,
//                                 );
//
//                                 let x_offset = (offset.x / self.horizontal_scale).round() as i32;
//                                 let y_offset = (offset.y / self.vertical_scale).round() as i32;
//
//                                 println!("{:?}, {:?}", original.tick, x_offset);
//
//                                 note.tick = max( 0, original.tick as i32 + x_offset) as u32;
//                                 note.note = max( 0, original.note as i32 + y_offset) as u8;
//                             }
//                         }
//                     },
//                     mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Pressed, } => {
//                         let hovered = self.notes.iter().enumerate()
//                             .find(|(_, note)| {
//                                 self.overlaps(self.cursor, note )
//                             });
//
//                         match hovered {
//                             None => {
//                                 let note = Note {
//                                     tick: (self.cursor.x / self.horizontal_scale) as u32,
//                                     note: (self.cursor.y / self.vertical_scale) as u8,
//                                     length: 2
//                                 };
//
//                                 self.notes.push(note);
//
//                                 self.dragging = Some((self.cursor.clone(), self.notes.len() - 1, note.clone()));
//                             },
//                             Some((idx, note)) => {
//                                 self.dragging = Some((self.cursor.clone(), idx, note.clone()));
//                             },
//                         }
//                     },
//                     mouse::Event::Input { button: mouse::Button::Left, state: ButtonState::Released, } => {
//                         self.dragging = None;
//                     },
//                     _ => {}
//                 }
//                 _ => {}
//             },
//             _ => {}
//         }
//
//         Command::none()
//     }
//
//
//     fn subscription(&self) -> Subscription<Message> {
//         iced_native::subscription::events().map(Message::Event)
//     }
//
//     fn view(&mut self) -> Element<Message> {
//         let children = self.notes.iter()
//             .map(|note| {
//                 (
//                     self.note_rect(note),
//                     NoteWidget::new().into(),
//                 )
//             })
//             .collect();
//
//         Column::new()
//             .push(Stack::with_children(children))
//             .push(Text::new(self.notes.iter().map(|n| { n.note as f32 + 3.2 * n.tick as f32 } ).sum::<f32>().to_string()))
//             .push(Button::new(&mut self.button, Text::new("wow")).on_press(Noop))
//             .into()
//     }
// }
//
//
