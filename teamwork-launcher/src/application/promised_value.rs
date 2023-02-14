use std::{fmt::Debug, hash::Hash};

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub enum PromisedValue<T: Clone + Hash + Debug> {
    Ready(T),
    Loading,
    None,
}

impl<T: Clone + Hash + Debug> PromisedValue<T> {
    pub fn is_none(&self) -> bool {
        matches!(self, PromisedValue::None)
    }
    pub fn is_ready(&self) -> bool {
        matches!(self, PromisedValue::Ready(_))
    }
    pub fn get(&self) -> Option<&T> {
        match self {
            PromisedValue::Ready(value) => Some(value),
            _ => None,
        }
    }
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
