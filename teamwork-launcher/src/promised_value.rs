use std::{fmt::Debug, hash::Hash};

#[derive(Debug, Hash, Clone)]
pub enum PromisedValue<T: Clone + Hash + Debug> {
    Ready(T),
    Loading,
    None,
}

impl<T: Clone + Hash + Debug> Default for PromisedValue<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T: Clone + Hash + Debug> From<Option<T>> for PromisedValue<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => PromisedValue::Ready(value),
            None => PromisedValue::None,
        }
    }
}
