use chrono::{Date, DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use clap::{ColorChoice, Parser, Subcommand};
use colour::e_yellow_ln;
use ctsettings::{ColdTurkeySettings, UserStatus};
use dialoguer::Password;
use std::io::Write;
use std::{fs::File, process};
use zeroize::Zeroizing;

mod blocksettings;
mod convert;
mod ctsettings;
mod historydeque;
mod loop_dialoguer;
mod matchstring;
mod suggestdialog;

#[derive(Parser)]
#[clap(
    name = "ctk",
    author = "Nguyen Tran (GitHub: ngtr6788)",
    version,
    color = ColorChoice::Never
)]
/// A better CLI interface for Cold Turkey.
///
/// Must have Cold Turkey installed in:
///     WINDOWS: C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe
///     MAC: /Applications/Cold Turkey Blocker.app/Contents/MacOS/Cold Turkey Blocker
struct ColdTurkey {
  #[clap(subcommand)]
  command: Option<Command>,
}

#[derive(Subcommand)]
enum StartSubcommands {
  /// Set a time period to block
  For {
    /// How long to block in minutes
    #[clap(long)]
    minutes: Option<u32>,
    #[clap(long)]
    hours: Option<u32>,
    #[clap(long)]
    days: Option<u32>,
  },
  /// Set when the block is finished
  Until {
    #[clap(parse(try_from_str = convert::str_to_time))]
    /// The time of the end of a block
    endtime: NaiveTime,
    #[clap(parse(try_from_str = convert::str_to_date))]
    /// The date of the end of a block. Defaults to today if not given
    enddate: Option<NaiveDate>,
  },
}

#[derive(Subcommand)]
enum Command {
  /// Start a block
  Start {
    /// The name of the Cold Turkey block
    block_name: String,
    #[clap(short, long)]
    /// Password to lock the block
    password: bool,
    #[clap(subcommand)]
    subcommand: Option<StartSubcommands>,
  },
  /// Stop a block
  Stop {
    /// The name of the Cold Turkey block
    block_name: String,
  },
  /// Add websites (urls) to a block
  Add {
    /// The name of the Cold Turkey block
    block_name: String,
    /// The url to add in the block
    url: String,
    #[clap(short, long)]
    /// Whether it is black or white-listed
    except: bool,
  },
  /// Turn on if off, turn off if on
  Toggle {
    /// The name of the Cold Turkey block
    block_name: String,
  },
  /// Interactively suggest what blocks you want Cold Turkey to have
  Suggest,
  /// List all the blocks in alphabetical order by default
  List,
  /// Installs Cold Turkey
  Install,
}

const CT_EXEC: &str = r"C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe";

const FROZEN_TURKEY: &str = "Frozen Turkey";

fn main() {
  let args = ColdTurkey::parse();
  match &args.command {
    Some(cmd) => match &cmd {
      Command::Start {
        block_name,
        password,
        subcommand,
      } => match password {
        true => start_block_with_password(block_name),
        false => match subcommand {
          Some(method) => match method {
            StartSubcommands::For {
              minutes,
              hours,
              days,
            } => {
              let total_minutes =
                days.unwrap_or(0) * 24 * 60 + hours.unwrap_or(0) * 60 + minutes.unwrap_or(0);
              start_block_for_some_minutes(block_name, total_minutes);
            }
            StartSubcommands::Until { endtime, enddate } => {
              start_block_until_time(block_name, *endtime, *enddate);
            }
          },
          None => start_block_unlocked(block_name),
        },
      },
      Command::Stop { block_name } => stop_block(block_name),
      Command::Add {
        block_name,
        url,
        except,
      } => add_websites_to_block(block_name, url, *except),
      Command::Toggle { block_name } => toggle_block(block_name),
      Command::Suggest => {
        suggestdialog::suggest();
      }
      Command::List => list_all_blocks(),
      Command::Install => install_cold_turkey(),
    },
    None => open_cold_turkey(),
  }
}

fn check_if_block_exists(block_name: &str) -> Option<bool> {
  if block_name == FROZEN_TURKEY {
    return Some(true);
  }

  let ct_settings = get_ct_settings();
  if let Some(settings) = &ct_settings {
    if settings.block_list_info.blocks.contains_key(block_name) {
      Some(true)
    } else {
      eprintln!(
        "ERROR: Block {} cannot be found in your Cold Turkey application",
        block_name
      );
      Some(false)
    }
  } else {
    eprintln!(
      "WARNING: ctk cannot check if block {} is in your Cold Turkey application right now",
      block_name
    );
    None
  }
}

fn start_block_with_password(block_name: &str) {
  if block_name == FROZEN_TURKEY {
    eprintln!("ERROR: You can only start Frozen Turkey when time is provided. Consider `ctk start for` or `ctk start until`.");
    return;
  }

  let ct_settings = get_ct_settings();
  if let Some(settings) = &ct_settings {
    if settings.is_pro == UserStatus::Free {
      eprintln!(
        "ERROR: Cannot start a block with a password as a free user. Consider upgrading to pro."
      );
      return;
    }

    if !settings.block_list_info.blocks.contains_key(block_name) {
      eprintln!(
        "ERROR: Block {} cannot be found in your Cold Turkey application",
        block_name
      );
      return;
    }
  } else {
    eprintln!(
      "WARNING: ctk cannot check if block {} is in your Cold Turkey application right now",
      block_name
    );
  }

  let p = Zeroizing::new(loop {
    match Password::new().with_prompt("Enter a password").interact() {
      Ok(pass) => break pass,
      Err(_) => continue,
    }
  });

  if process::Command::new(CT_EXEC)
    .args(["-start", block_name, "-password", &p])
    .spawn()
    .is_ok()
  {
    eprintln!("SUCCESS: Starts blocking {} with a password", block_name);
  } else {
    eprintln!("ERROR: Cannot run `ctk start --password`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
  }
}

fn start_block_for_some_minutes(block_name: &str, minutes: u32) {
  if Some(false) == check_if_block_exists(block_name) {
    return;
  }

  if process::Command::new(CT_EXEC)
    .args(["-start", block_name, "-lock", &minutes.to_string()])
    .spawn()
    .is_ok()
  {
    eprintln!(
      "SUCCESS: Starts blocking {} locked for {} minutes",
      block_name, minutes
    );
  } else {
    eprintln!("ERROR: Cannot run `ctk start for`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
  }
}

fn start_block_until_time(block_name: &str, endtime: NaiveTime, enddate: Option<NaiveDate>) {
  if Some(false) == check_if_block_exists(block_name) {
    return;
  }

  let datetime: DateTime<Local> = match enddate {
    Some(date) => {
      let naive_datetime: NaiveDateTime = date.and_time(endtime);
      let datetime_result: LocalResult<DateTime<Local>> =
        Local.from_local_datetime(&naive_datetime);
      match datetime_result {
        LocalResult::None => {
          eprintln!("ERROR: Can't get the datetime specified.");
          return;
        }
        LocalResult::Single(datetime) => datetime,
        LocalResult::Ambiguous(_, _) => {
          eprintln!("ERROR: Datetime given is ambiguous. Maybe try to be more clear in your time?");
          return;
        }
      }
    }
    None => {
      let today: Date<Local> = Local::today();
      let today_time_option: Option<DateTime<Local>> = today.and_time(endtime);
      match today_time_option {
        Some(datetime) => datetime,
        None => {
          eprintln!("ERROR: The date is assumed to be today, however, the time given seems to make it invalid.");
          return;
        }
      }
    }
  };

  let duration = datetime.signed_duration_since(Local::now());
  // If duration is exactly a multiple of 60, do not round up
  let duration_minutes = if duration.num_seconds() % 60 == 0 {
    duration.num_minutes()
  } else {
    duration.num_minutes() + 1
  };

  if duration_minutes <= 0 {
    eprintln!(
      "ERROR: Cannot start block until a time in the past. Please enter a time in the future."
    );
    return;
  }

  if process::Command::new(CT_EXEC)
    .args(["-start", block_name, "-lock", &duration_minutes.to_string()])
    .spawn()
    .is_ok()
  {
    eprintln!(
      "SUCCESS: Starts blocking {} locked until {}",
      block_name,
      datetime.format("%H:%M %B %d %Y")
    );
  } else {
    eprintln!("ERROR: Cannot run `ctk start until`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
  }
}

fn start_block_unlocked(block_name: &str) {
  if block_name == FROZEN_TURKEY {
    eprintln!("ERROR: You can only start Frozen Turkey when time is provided. Consider `ctk start for` or `ctk start until`.");
    return;
  }

  if Some(false) == check_if_block_exists(block_name) {
    return;
  }

  if process::Command::new(CT_EXEC)
    .args(["-start", block_name])
    .spawn()
    .is_ok()
  {
    eprintln!("SUCCESS: Starts blocking {}", block_name);
  } else {
    eprintln!("ERROR: Cannot run `ctk start`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
  }
}

fn stop_block(block_name: &str) {
  if FROZEN_TURKEY == block_name {
    eprintln!("FAILURE: Cannot stop Frozen Turkey because it is a timed, locked block. If it is already off, no need to worry.");
    return;
  }

  if Some(false) == check_if_block_exists(block_name) {
    return;
  }

  if process::Command::new(CT_EXEC)
    .args(["-stop", block_name])
    .spawn()
    .is_ok()
  {
    // Why unwrap? It's safe to assume that if the first get_ct_settings works, why not the second?
    let new_ct_settings = get_ct_settings().unwrap();
    if new_ct_settings.block_list_info.blocks[block_name].is_dormant() {
      eprintln!("SUCCESS: Stops blocking {}", block_name);
    } else {
      eprintln!("FAILURE: Failed to stop blocking {block_name}");
    }
  } else {
    eprintln!("ERROR: Cannot run `ctk stop`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
  }
}

fn add_websites_to_block(block_name: &str, url: &str, except: bool) {
  if block_name == FROZEN_TURKEY {
    eprintln!("ERROR: You cannot add websites to the Frozen Turkey block.");
    return;
  }

  if Some(false) == check_if_block_exists(block_name) {
    return;
  }

  let except_cmd: &str = if except { "-exception" } else { "-web" };
  if process::Command::new(CT_EXEC)
    .args(["-add", block_name, except_cmd, url])
    .spawn()
    .is_ok()
  {
    let new_ct_settings = get_ct_settings().unwrap();
    let block_info = &new_ct_settings.block_list_info.blocks[block_name];

    let is_dormant = block_info.is_dormant();

    let string_url = url.to_string();
    if except && (is_dormant || block_info.exception_list.contains(&string_url)) {
      eprintln!("SUCCESS: Adds url {url} to block {block_name} as an exception");
    } else if !except && (is_dormant || block_info.block_list.contains(&string_url)) {
      eprintln!("SUCCESS: Adds url {url} to block {block_name}");
    } else {
      eprintln!("FAILURE: Cannot add url {url} to block {block_name}");
    }
  } else {
    eprintln!("ERROR: Cannot run `ctk add`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
  }
}

fn toggle_block(block_name: &str) {
  if block_name == FROZEN_TURKEY {
    eprintln!("ERROR: You can only start Frozen Turkey when time is provided. Consider `ctk start for` or `ctk start until`.");
    return;
  }

  let ct_settings = get_ct_settings();
  let is_off_old = if let Some(settings) = &ct_settings {
    if settings.block_list_info.blocks.contains_key(block_name) {
      settings.block_list_info.blocks[block_name].is_dormant()
    } else {
      eprintln!(
        "ERROR: Block {} cannot be found in your Cold Turkey application",
        block_name
      );
      return;
    }
  } else {
    eprintln!(
      "WARNING: ctk cannot check if block {} is in your Cold Turkey application right now",
      block_name
    );
    return;
  };

  if process::Command::new(CT_EXEC)
    .args(["-toggle", block_name])
    .spawn()
    .is_ok()
  {
    // Why unwrap? It's safe to assume that if the first get_ct_settings works, why not the second?
    let new_settings = get_ct_settings().unwrap();
    let is_off_new = new_settings.block_list_info.blocks[block_name].is_dormant();

    match (is_off_old, is_off_new) {
      (true, false) => eprintln!("SUCCESS: Starts block {}", block_name),
      (false, true) => eprintln!("SUCCESS: Stops block {block_name}"),
      (false, false) => eprintln!("FAILURE: Failed to stop blocking {block_name}"),
      (true, true) => eprintln!("FAILURE: Failed to start blocking {block_name}"), // rare, probs impossible
    }
  } else {
    eprintln!("ERROR: Cannot run `ctk toggle`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
  }
}

fn open_cold_turkey() {
  if process::Command::new(CT_EXEC).spawn().is_ok() {
    eprintln!("SUCCESS: Launches Cold Turkey!");
  } else {
    eprintln!(
      r"ERROR: Looks like you don't have Cold Turkey installed on C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe"
    );
    eprintln!(
      "If you do have it installed, please put Cold Turkey Blocker.exe in the folder mentioned."
    );
    eprintln!("If not, you're welcome to download it at getcoldturkey.com.");
  }
}

fn list_all_blocks() {
  let ct_settings = get_ct_settings();
  if let Some(settings) = ct_settings {
    let keys = settings.block_list_info.blocks.keys();
    let mut sorted_keys = Vec::new();
    for key in keys {
      let block_inactive = settings.block_list_info.blocks[key].is_dormant();
      sorted_keys.push((key, block_inactive));
    }

    sorted_keys.sort_unstable();
    for (key, inactive) in sorted_keys {
      if inactive {
        println!("{}", key);
      } else {
        e_yellow_ln!("* {} *", key);
      }
    }
  } else {
    eprintln!("ERROR: ctk cannot determine all the blocks right now");
  }
}

fn get_ct_settings() -> Option<ColdTurkeySettings> {
  match process::Command::new(r"C:\Program Files\Cold Turkey\CTMsgHostEdge.exe").output() {
    Ok(block_stdout) => {
      let output_vector = block_stdout.stdout;
      match std::str::from_utf8(&output_vector[4..]) {
        Ok(ct_string) => serde_json::from_str(ct_string).ok(),
        Err(_) => None,
      }
    }
    Err(_) => None,
  }
}

fn install_cold_turkey() {
  match try_install_cold_turkey() {
    Ok(_) => eprintln!("SUCCESS: Installation successful"),
    Err(_) => eprintln!("ERROR: Something went wrong in downloading the Cold Turkey installer."),
  }
}

fn try_install_cold_turkey() -> Result<(), Box<dyn std::error::Error>> {
  let url = "http://getcoldturkey.com/files/Cold_Turkey_Installer.exe";
  let response = reqwest::blocking::get(url)?;
  {
    let mut file = File::create("Cold_Turkey_Installer.exe")?;
    let bytes = response.bytes()?;
    file.write_all(&bytes)?;
  }
  process::Command::new("./Cold_Turkey_Installer.exe").spawn()?;

  Ok(())
}
