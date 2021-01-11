use crate::sequence::Pitch;

pub struct Event {
    pub sample: usize,
    pub sequence: usize,
    pub data: EventData,
}

pub enum EventData {
    NoteOn(u32, Pitch),
    NoteOff(u32, Pitch),
    ClearEvents,
}

pub trait Controller {
    fn send_event(&self, event: Event);
}
