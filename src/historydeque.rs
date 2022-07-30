use dialoguer::History;
use std::{collections::VecDeque, fmt::Display};

pub struct HistoryDeque<T> {
  deque: VecDeque<T>,
}

impl<T> HistoryDeque<T> {
  pub fn new() -> Self {
    HistoryDeque {
      deque: VecDeque::<T>::new(),
    }
  }
}

impl<T> History<T> for HistoryDeque<T>
where
  T: Display + Clone,
{
  fn read(&self, pos: usize) -> Option<String> {
    if let Some(val) = self.deque.get(pos) {
      Some(val.to_string())
    } else {
      None
    }
  }

  fn write(&mut self, val: &T) {
    self.deque.push_front(val.clone());
  }
}
