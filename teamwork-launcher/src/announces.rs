use std::collections::VecDeque;

pub struct Announce {
    pub title: String,
    pub message: String,
}

impl Announce {
    pub fn new(title: impl ToString, message: impl ToString) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
        }
    }
}

pub struct AnnounceQueue {
    queue: VecDeque<Announce>,
}

impl Default for AnnounceQueue {
    fn default() -> Self {
        Self { queue: VecDeque::new() }
    }
}

impl AnnounceQueue {
    pub fn push(&mut self, announce: Announce) {
        self.queue.push_front(announce)
    }

    pub fn pop(&mut self) {
        self.queue.pop_back();
    }

    pub fn current(&self) -> Option<&Announce> {
        self.queue.back()
    }
}
