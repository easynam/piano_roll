
pub type Sequence = Vec<Note>;

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub(crate) tick: i32,
    pub(crate) note: u8,
    pub(crate) length: i32,
}

#[derive(Debug, Clone)]
pub enum SequenceChange {
    Add(Note),
    Remove(usize),
    Update(usize, Note),
}

pub fn update_sequence(seq: &mut Sequence, message: SequenceChange) {
    match message {
        SequenceChange::Add(note) => {
            seq.push(note);
        },
        SequenceChange::Remove(idx) => {
            seq.remove(idx);
        },
        SequenceChange::Update(idx, note) => {
            seq[idx] = note;
        },
    }
}