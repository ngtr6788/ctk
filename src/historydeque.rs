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
    self.deque.get(pos).map(std::string::ToString::to_string)
  }

  fn write(&mut self, val: &T) {
    self.deque.push_front(val.clone());
  }
}

impl<T> Default for HistoryDeque<T> {
  fn default() -> Self {
    Self::new()
  }
}
