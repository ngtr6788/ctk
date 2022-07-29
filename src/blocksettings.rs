use chrono::{NaiveTime, Timelike};
use serde::{Serialize, Serializer};

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BlockSettings {
  #[serde(rename = "type")]
  pub sched_type: SchedType,
  pub lock: LockMethod,
  #[serde(serialize_with = "bool_str_serialize")]
  pub lock_unblock: bool,
  #[serde(serialize_with = "bool_str_serialize")]
  pub restart_unblock: bool,
  pub password: String,
  #[serde(serialize_with = "u16_str_serialize")]
  pub random_text_length: u16,
  #[serde(rename = "break")]
  pub break_type: BreakMethod,
  pub window: RangeWindow,
  pub users: String,
  pub web: Vec<String>,
  pub exceptions: Vec<String>,
  pub apps: Vec<AppString>,
  pub schedule: Vec<ScheduleBlock>,
  pub custom_users: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum BreakMethod {
  None,
  Allowance(u8),
  Pomodoro(u8, u8),
}

impl Serialize for BreakMethod {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      BreakMethod::None => serializer.serialize_str("none"),
      BreakMethod::Allowance(allow) => {
        let allow_str = allow.to_string();
        serializer.serialize_str(&allow_str)
      }
      BreakMethod::Pomodoro(block_min, break_min) => {
        let pomodoro_str = format!("{},{}", block_min, break_min);
        serializer.serialize_str(&pomodoro_str)
      }
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ScheduleBlock {
  #[serde(serialize_with = "usize_str_serialize")]
  pub id: usize,
  pub start_time: ScheduleTimeTuple,
  pub end_time: ScheduleTimeTuple,
  #[serde(rename = "break")]
  pub break_type: BreakMethod,
}

// #[derive(Debug)]
// pub struct ScheduleTime {
//   pub day_of_week: Day,
//   pub time: NaiveTime,
// }

// #[derive(Debug, Copy, Clone)]
// pub enum Day {
//   Sun,
//   Mon,
//   Tue,
//   Wed,
//   Thu,
//   Fri,
//   Sat,
// }

// impl Serialize for ScheduleTime {
//   fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//     let day_int: u8 = match &self.day_of_week {
//       Day::Sun => 0,
//       Day::Mon => 1,
//       Day::Tue => 2,
//       Day::Wed => 3,
//       Day::Thu => 4,
//       Day::Fri => 5,
//       Day::Sat => 6,
//     };

//     let schedule_time_str = format!("{},{},{}", day_int, self.time.hour(), self.time.minute());

//     serializer.serialize_str(&schedule_time_str)
//   }
// }

#[derive(Debug)]
pub struct ScheduleTimeTuple(usize, u32, u32);

impl ScheduleTimeTuple {
  // It's a tuple, why bother with new? It's for better communication.
  pub fn new(day_of_week: usize, hour: u32, minute: u32) -> Self {
    ScheduleTimeTuple(day_of_week, hour, minute)
  }
}

impl Serialize for ScheduleTimeTuple {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let sched_str = format!("{},{},{}", self.0, self.1, self.2);

    serializer.serialize_str(&sched_str)
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LockMethod {
  None,
  RandomText,
  Window,
  Restart,
  Password,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum SchedType {
  Continuous,
  Scheduled,
}

#[derive(Debug)]
pub enum AppString {
  File(String),
  Folder(String),
  Win10(String),
  Title(String),
}

#[derive(Debug)]
pub struct RangeWindow {
  pub lock_range: bool,
  pub start_time: NaiveTime,
  pub end_time: NaiveTime,
}

impl Serialize for RangeWindow {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let range_str = format!(
      "{}lock@{},{}@{},{}",
      if self.lock_range { "" } else { "un" },
      self.start_time.hour(),
      self.start_time.minute(),
      self.end_time.hour(),
      self.end_time.minute()
    );

    serializer.serialize_str(&range_str)
  }
}

impl Serialize for AppString {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let app_string: String = match self {
      AppString::File(path) => {
        format!("file:{}", path.replace("\\", "/"))
      }
      AppString::Folder(path) => {
        format!("folder:{}", path.replace("\\", "/"))
      }
      AppString::Win10(string) => {
        format!("win10:{}", string)
      }
      AppString::Title(string) => {
        format!("title:{}", string)
      }
    };

    serializer.serialize_str(&app_string)
  }
}

impl BlockSettings {
  pub fn new() -> Self {
    let new_settings: BlockSettings = BlockSettings {
      sched_type: SchedType::Continuous,
      lock: LockMethod::None,
      lock_unblock: true,
      restart_unblock: true,
      password: String::new(),
      random_text_length: 30,
      break_type: BreakMethod::None,
      window: RangeWindow {
        lock_range: true,
        start_time: NaiveTime::from_hms(9, 0, 0),
        end_time: NaiveTime::from_hms(17, 0, 0),
      },
      users: String::new(),
      web: Vec::new(),
      exceptions: vec!["file://*".to_string()],
      apps: Vec::new(),
      schedule: Vec::new(),
      custom_users: Vec::new(),
    };
    new_settings
  }
}

impl Default for BlockSettings {
  fn default() -> Self {
    Self::new()
  }
}

fn bool_str_serialize<S: Serializer>(my_bool: &bool, serializer: S) -> Result<S::Ok, S::Error> {
  let bool_str = my_bool.to_string();
  serializer.serialize_str(&bool_str)
}

fn u16_str_serialize<S: Serializer>(num: &u16, serializer: S) -> Result<S::Ok, S::Error> {
  let u16_str = num.to_string();
  serializer.serialize_str(&u16_str)
}

fn usize_str_serialize<S: Serializer>(num: &usize, serializer: S) -> Result<S::Ok, S::Error> {
  let usize_str = num.to_string();
  serializer.serialize_str(&usize_str)
}
