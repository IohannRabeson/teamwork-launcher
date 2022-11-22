#[derive(PartialEq, Eq)]
pub enum States {
    Normal,
    Favorites,
    Settings,
    Reloading,
    Error { message: String },
}

pub struct StatesStack {
    states: Vec<States>,
}

impl StatesStack {
    pub fn new(initial: States) -> Self {
        Self { states: vec![initial] }
    }

    pub fn current(&self) -> &States {
        self.states.last().expect("states must never be empty")
    }

    pub fn reset(&mut self, state: States) {
        self.states.clear();
        self.states.push(state);
    }

    pub fn push(&mut self, state: States) {
        self.states.push(state);
    }

    pub fn pop(&mut self) {
        self.states.pop();
    }
}
