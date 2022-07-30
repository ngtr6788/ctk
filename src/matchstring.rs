use sublime_fuzzy::Match;

pub struct MatchString {
  pub match_object: Match,
  pub string: String,
}

impl Ord for MatchString {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.match_object.cmp(&other.match_object)
  }
}

impl Eq for MatchString {}

impl PartialOrd for MatchString {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.match_object.partial_cmp(&other.match_object)
  }
}

impl PartialEq for MatchString {
  fn eq(&self, other: &Self) -> bool {
    self.match_object.eq(&other.match_object)
  }
}

impl ToString for MatchString {
  fn to_string(&self) -> String {
    self.string.clone()
  }
}
