
use memo::Memo;
use std::collections::VecDeque;

pub type SubjectId     = u64;
pub type SubjectField  = String;

pub struct SubjectHead {
    id:    SubjectId,
    head:  Vec<Memo>
}

pub struct SubjectMemoIter {
    head: Vec<Memo>,
    next: VecDeque<Memo>,
    context: Context
}

impl SubjectMemoIter {
    pub fn new (head: &[&Memo]) -> Self {
        SubjectMemoIter {
            head: head,
            next: VecDeque::new()
        }
    }
}

impl Iterator for SubjectMemoIter {
    fn next (&mut self) -> Option<Memo> {

        for memo in self.head {

        }

    }
}
