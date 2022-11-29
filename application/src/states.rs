
/// A stack of states.
/// S is the type of the state stored.
/// Is this component generic enough to be moved into a dedicated crate?
pub struct StatesStack<S> {
    states: Vec<S>,
}

impl<S> StatesStack<S> {
    pub fn new(initial: S) -> Self {
        Self { states: vec![initial] }
    }

    pub fn current(&self) -> &S {
        self.states.last().expect("states must never be empty")
    }

    pub fn reset(&mut self, state: S) {
        self.states.clear();
        self.states.push(state);
    }

    pub fn push(&mut self, state: S) {
        self.states.push(state);
    }

    pub fn pop(&mut self) {
        self.states.pop();
    }
}
