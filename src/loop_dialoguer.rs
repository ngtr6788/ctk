
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use std::{fmt::Debug, io, str::FromStr};

fn looped_dialoguer<F, S>(mut dialogue: F) -> S
where
  F: FnMut() -> io::Result<S>,
{
  loop {
    match dialogue() {
      Ok(value) => break value,
      Err(err) => {
        eprintln!("{}", err);
        continue;
      }
    }
  }
}

pub trait LoopDialogue<S> {
  fn loop_interact(&mut self) -> S;
}

impl LoopDialogue<bool> for Confirm<'_> {
  fn loop_interact(&mut self) -> bool {
    looped_dialoguer(|| self.interact())
  }
}

impl<S> LoopDialogue<S> for Input<'_, S>
where
  S: Clone + ToString + FromStr,
  <S as FromStr>::Err: Debug + ToString,
{
  fn loop_interact(&mut self) -> S {
    looped_dialoguer(|| self.interact_text())
  }
}

impl LoopDialogue<Vec<usize>> for MultiSelect<'_> {
  fn loop_interact(&mut self) -> Vec<usize> {
    looped_dialoguer(|| self.interact())
  }
}

impl LoopDialogue<String> for Password<'_> {
  fn loop_interact(&mut self) -> String {
    looped_dialoguer(|| self.interact())
  }
}

impl LoopDialogue<usize> for Select<'_> {
  fn loop_interact(&mut self) -> usize {
    looped_dialoguer(|| self.interact())
  }
}
