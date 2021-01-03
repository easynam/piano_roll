use crate::sequence::Note;

pub struct Event {
    pub sample: usize,
    pub data: EventData,
}

pub enum EventData {
    NoteOn(Note),
    NoteOff(Note),
}

pub trait Controller {
    fn send_event(&self, event: Event);
}
