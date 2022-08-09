use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdTurkeySettings {
  #[serde(deserialize_with = "deserialize_string_to_u32")]
  pub version: u32,
  pub block_list_info: BlockListInfo,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub paused: bool,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub stats_enabled: bool,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub stats_enabled_incognito: bool,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub stats_strict: bool,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub block_embedded: bool,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub block_charity: bool,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub block_inactive: bool,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub force_allow_file: bool,
  pub is_pro: String,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub ignore_incognito: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockListInfo {
  pub blocks: HashMap<String, BlockInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
  #[serde(deserialize_with = "deserialize_string_to_option_u32")]
  pub allowance: Option<u32>,
  #[serde(deserialize_with = "deserialize_string_to_option_u32")]
  pub allowance_remaining: Option<u32>,
  #[serde(deserialize_with = "deserialize_string_to_option_u32")]
  pub pomodoro_period_remaining: Option<u32>,
  pub pomodoro_period_state: String,
  pub password: String,
  #[serde(deserialize_with = "deserialize_string_to_option_u32")]
  pub random_text_length: Option<u32>,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub schedule_show_all: bool,
  pub block_list: Vec<String>,
  pub exception_list: Vec<String>,
  pub title_list: Vec<String>,
}

impl BlockInfo {
  pub fn is_dormant(&self) -> bool {
    self.allowance == None
    && self.allowance_remaining == None
    && self.pomodoro_period_remaining == None
    && self.pomodoro_period_state.is_empty()
    && self.password.is_empty()
    && self.random_text_length == None
    && self.block_list.is_empty()
    && self.exception_list.is_empty()
    && self.title_list.is_empty()
  }
}

fn deserialize_string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  let s: &str = Deserialize::deserialize(deserializer)?;
  match s {
    "true" => Ok(true),
    "false" => Ok(false),
    _ => Err(Error::unknown_variant(s, &["true", "false"])),
  }
}

fn deserialize_string_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
  D: Deserializer<'de>,
{
  let s: &str = Deserialize::deserialize(deserializer)?;
  match s.parse::<u32>() {
    Ok(num) => Ok(num),
    Err(_) => Err(Error::invalid_type(
      Unexpected::Str(s),
      &"not a u32 integer",
    )),
  }
}

fn deserialize_string_to_option_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error> 
where
    D: Deserializer<'de>
{
  let s: &str = Deserialize::deserialize(deserializer)?;
  if s.is_empty() {
    return Ok(None);
  }
  match s.parse::<u32>() {
    Ok(num) => Ok(Some(num)),
    Err(_) => Err(Error::invalid_type(
      Unexpected::Str(s),
      &"not a u32 integer",
    )),
  }
}
