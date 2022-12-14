use std::collections::VecDeque;

#[derive(PartialEq, Eq, Debug, Clone)]
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
    /// Enqueue a new announce on the display queue.
    /// The announce is only added if it's not already present in the queue.
    pub fn push(&mut self, announce: Announce) {
        if !self.queue.contains(&announce) {
            self.queue.push_front(announce);
        }
    }

    pub fn pop(&mut self) {
        self.queue.pop_back();
    }

    pub fn current(&self) -> Option<&Announce> {
        self.queue.back()
    }
}
