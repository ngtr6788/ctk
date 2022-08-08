// Get rid of unused_must_use errors for now
#![allow(unused_must_use)]
use chrono::{Date, DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use clap::{ColorChoice, Parser, Subcommand};
use ctsettings::ColdTurkeySettings;
use dialoguer::Password;
use std::process;
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
enum ForSubcommands {
  /// Set a time period to block
  For {
    /// How long to block in minutes
    minutes: u32,
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
    subcommand: Option<ForSubcommands>,
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
}

fn main() {
  let mut cold_turkey =
    process::Command::new(r"C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe");

  let ct_settings: Option<ColdTurkeySettings> =
    match process::Command::new(r"C:\Program Files\Cold Turkey\CTMsgHostEdge.exe").output() {
      Ok(block_stdout) => {
        let output_vector = block_stdout.stdout;
        match std::str::from_utf8(&output_vector[4..]) {
          Ok(ct_string) => serde_json::from_str(ct_string).ok(),
          Err(_) => None,
        }
      }
      Err(_) => None,
    };

  let check_block_exists_then_succeed = |block_name: &str, success_message: String| {
    if let Some(settings) = &ct_settings {
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
    eprintln!("{}", success_message);
  };

  let args = ColdTurkey::parse();
  match &args.command {
    Some(cmd) => match &cmd {
      Command::Start {
        block_name,
        password,
        subcommand,
      } => {
        cold_turkey.args(["-start", block_name]);
        match password {
          true => {
            if let Some(settings) = &ct_settings {
              if settings.is_pro == "free" {
                eprintln!("ERROR: Cannot start a block with a password as a free user. Consider upgrading to pro.");
                return;
              }
            } else {
              eprintln!("WARNING: Cannot check if user is a pro user or not right now.");
            }

            let p = Zeroizing::new(loop {
              match Password::new().with_prompt("Enter a password").interact() {
                Ok(pass) => break pass,
                Err(_) => continue,
              }
            });

            match cold_turkey.args(["-password", &p]).spawn() {
              Ok(_) => {
                check_block_exists_then_succeed(
                  block_name,
                  format!("SUCCESS: Starts blocking {} with a password", block_name),
                );
              }
              Err(_) => {
                eprintln!("ERROR: Cannot run `ctk start --password`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
              }
            };
          }
          false => match subcommand {
            Some(method) => match method {
              ForSubcommands::For { minutes } => {
                match cold_turkey.args(["-lock", &minutes.to_string()]).spawn() {
                  Ok(_) => {
                    check_block_exists_then_succeed(
                      block_name,
                      format!(
                        "SUCCESS: Starts blocking {} locked for {} minutes",
                        block_name, minutes
                      ),
                    );
                  }
                  Err(_) => {
                    eprintln!("ERROR: Cannot run `ctk start for`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                  }
                };
              }
              ForSubcommands::Until { endtime, enddate } => {
                let datetime: DateTime<Local> = match enddate {
                  Some(date) => {
                    let naive_datetime: NaiveDateTime = date.and_time(*endtime);
                    let datetime_result: LocalResult<DateTime<Local>> =
                      Local.from_local_datetime(&naive_datetime);
                    match datetime_result {
                      LocalResult::None => {
                        eprintln!("ERROR: Can't get the datetime specified.");
                        return;
                      }
                      LocalResult::Single(datetime) => datetime,
                      LocalResult::Ambiguous(_, _) => {
                        eprintln!(
                            "ERROR: Datetime given is ambiguous. Maybe try to be more clear in your time?"
                          );
                        return;
                      }
                    }
                  }
                  None => {
                    let today: Date<Local> = Local::today();
                    let today_time_option: Option<DateTime<Local>> = today.and_time(*endtime);
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
                match cold_turkey
                  .args(["-lock", &duration_minutes.to_string()])
                  .spawn()
                {
                  Ok(_) => {
                    check_block_exists_then_succeed(
                      block_name,
                      format!(
                        "SUCCESS: Starts blocking {} locked until {}",
                        block_name,
                        datetime.format("%H:%M %B %d %Y")
                      ),
                    );
                  }
                  Err(_) => {
                    eprintln!("ERROR: Cannot run `ctk start until`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                  }
                };
              }
            },
            None => {
              match cold_turkey.spawn() {
                Ok(_) => {
                  check_block_exists_then_succeed(
                    block_name,
                    format!("SUCCESS: Starts blocking {}", block_name),
                  );
                }
                Err(_) => {
                  eprintln!("ERROR: Cannot run `ctk start`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                }
              };
            }
          },
        }
      }
      Command::Stop { block_name } => {
        match cold_turkey.args(["-stop", block_name]).spawn() {
          Ok(_) => {
            check_block_exists_then_succeed(
              block_name,
              format!("SUCCESS: Stops blocking {}", block_name),
            );
          }
          Err(_) => {
            eprintln!("ERROR: Cannot run `ctk stop`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
          }
        };
      }
      Command::Add {
        block_name,
        url,
        except,
      } => {
        let except_cmd: &str = if *except { "-exception" } else { "-web" };
        match cold_turkey
          .args(["-add", block_name, except_cmd, url])
          .spawn()
        {
          Ok(_) => {
            check_block_exists_then_succeed(
              block_name,
              format!("SUCCESS: Adds url {} to block {}", url, block_name),
            );
          }
          Err(_) => {
            eprintln!("ERROR: Cannot run `ctk add`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
          }
        }
      }
      Command::Toggle { block_name } => {
        match cold_turkey.args(["-toggle", block_name]).spawn() {
          Ok(_) => {
            check_block_exists_then_succeed(
              block_name,
              format!("SUCCESS: Toggles block {}", block_name),
            );
          }
          Err(_) => {
            eprintln!("ERROR: Cannot run `ctk toggle`. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
          }
        };
      }
      Command::Suggest => {
        suggestdialog::suggest();
      }
    },
    None => {
      if cold_turkey.spawn().is_ok() {
        eprintln!("SUCCESS: Launches Cold Turkey!");
      } else {
        eprintln!(
          r"ERROR: Looks like you don't have Cold Turkey installed on C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe"
        );
        eprintln!("If you do have it installed, please put Cold Turkey Blocker.exe in the folder mentioned.");
        eprintln!("If not, you're welcome to download it at getcoldturkey.com.");
      }
    }
  }
}
