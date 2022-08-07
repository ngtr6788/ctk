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
  #[serde(deserialize_with = "deserialize_string_to_u32")]
  pub allowance: u32,
  #[serde(deserialize_with = "deserialize_string_to_u32")]
  pub allowance_remaining: u32,
  #[serde(deserialize_with = "deserialize_string_to_u32")]
  pub pomodoro_period_remaining: u32,
  pub pomodoro_period_state: String,
  pub password: String,
  #[serde(deserialize_with = "deserialize_string_to_u32")]
  pub random_text_length: u32,
  #[serde(deserialize_with = "deserialize_string_to_bool")]
  pub schedule_show_all: bool,
  pub block_list: Vec<String>,
  pub exception_list: Vec<String>,
  pub title_list: Vec<String>,
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
  if s.is_empty() {
    return Ok(0);
  }
  match s.parse::<u32>() {
    Ok(num) => Ok(num),
    Err(_) => Err(Error::invalid_type(
      Unexpected::Str(s),
      &"not a u32 integer",
    )),
  }
}
