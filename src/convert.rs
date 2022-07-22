use chrono::{NaiveDate, NaiveTime, ParseResult};

pub fn str_to_time(s: &str) -> ParseResult<NaiveTime> {
  const ALLOWED_PARSE: [&str; 6] = ["%H:%M", "%k:%M", "%I:%M%P", "%I:%M%p", "%l:%M%P", "%l:%M%p"];
  for parser in &ALLOWED_PARSE {
    match NaiveTime::parse_from_str(s, parser) {
      Ok(time) => return Ok(time),
      Err(_) => continue,
    }
  }
  return NaiveTime::parse_from_str(s, ALLOWED_PARSE[0]);
}

pub fn str_to_date(s: &str) -> ParseResult<NaiveDate> {
  const ALLOWED_PARSE: [&str; 6] = [
    "%d %B %Y", "%e %B %Y", "%B %d %Y", "%B %e %Y", "%F", "%d/%m/%Y",
  ];
  for parser in &ALLOWED_PARSE {
    match NaiveDate::parse_from_str(s, parser) {
      Ok(time) => return Ok(time),
      Err(_) => continue,
    }
  }
  return NaiveDate::parse_from_str(s, ALLOWED_PARSE[0]);
}
