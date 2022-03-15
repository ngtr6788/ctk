#![allow(unused_must_use, unused_variables, unused_imports)]

/**
 * ctk is essentially a rewrite of my previous project,
 * pyturkey, which is an improved CLI interface for Cold Turkey.
 *
 * I basically rewrite this whole thing in Rust, I guess
 */

/*
ctk, a better CLI interface for Cold Turkey

Usage:
  ctk
  ctk start <block_name>
  ctk start <block_name> for <minutes>
  ctk start <block_name> until <time> [<date>]
  ctk stop <block_name>
  ctk add <block_name> <url> [-e]
  ctk toggle <block_name>
  ctk suggest

Options:
  -h --help         Show this screen.
  -e --except       Adds <url> as an exception
*/

use std::process;
use chrono::{Date, DateTime, NaiveTime, NaiveDate, NaiveDateTime, ParseResult, TimeZone, Local, LocalResult};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(
  name = "ctk",
  author = "Nguyen Tran (GitHub: ngtr6788)",
  version = "0.0.1",
)]
/// A better CLI interface for Cold Turkey
struct ColdTurkey {
  #[clap(subcommand)]
  command: Option<Command>,
}

#[derive(Subcommand)]
enum ForSubcommands {
  /// Set a time period to block
  For {
    /// How long to block in minutes
    minutes: u32
  },
  /// Set when the block is finished
  Until {
    #[clap(parse(try_from_str = str_to_time))]
    /// The time of the end of a block
    endtime: NaiveTime,
    #[clap(parse(try_from_str = str_to_date))]
    /// The date of the end of a block. Defaults to today if not given
    enddate: Option<NaiveDate>,
  }
}

fn str_to_time(s: &str) -> ParseResult<NaiveTime> {
  const ALLOWED_PARSE: [&str; 6] = ["%H:%M", "%k:%M", "%I:%M%P", "%I:%M%p", "%l:%M%P", "%l:%M%p"];
  for parser in &ALLOWED_PARSE {
    match NaiveTime::parse_from_str(s, parser) {
      Ok(time) => return Ok(time),
      Err(_) => continue,
    }
  }
  return NaiveTime::parse_from_str(s, ALLOWED_PARSE[0]);
}

fn str_to_date(s: &str) -> ParseResult<NaiveDate> {
  const ALLOWED_PARSE: [&str; 5] = ["%d %B %Y", "%e %B %Y", "%B %d %Y, %B %e %Y", "%F", "%d/%m/%Y"];
  for parser in &ALLOWED_PARSE {
    match NaiveDate::parse_from_str(s, parser) {
      Ok(time) => return Ok(time),
      Err(_) => continue,
    }
  }
  return NaiveDate::parse_from_str(s, ALLOWED_PARSE[0]);
}

#[derive(Subcommand)]
enum Command {
  /// Start a block
  Start {
    /// The name of the Cold Turkey block
    block_name: String,
    #[clap(short, long)]
    /// Password to lock the block
    password: Option<String>,
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
  /// Suggest settings for Cold Turkey .ctbbl files with commands
  Suggest,
}

fn main() {
  let mut cold_turkey = process::Command::new(r"C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe");
  let args = ColdTurkey::parse();
  match &args.command {
    Some(cmd) => {
      match &cmd {
        Command::Start{ block_name, password, subcommand } => {
          cold_turkey.args(["-start", block_name]);
          match password {
            Some(p) => {
              cold_turkey.args(["-password", p]).spawn();
            },
            None => {
              match subcommand {
                Some(method) => {
                  match method {
                    ForSubcommands::For{ minutes } => {
                      cold_turkey.args(["-lock", &minutes.to_string()]).spawn();
                    },
                    ForSubcommands::Until{ endtime, enddate } => {
                      let datetime: DateTime<Local> = match enddate {
                        Some(date) => {
                          let naive_datetime: NaiveDateTime = date.and_time(*endtime);
                          let datetime_result: LocalResult<DateTime<Local>> = Local.from_local_datetime(&naive_datetime);
                          match datetime_result {
                            LocalResult::None => {
                              println!("Can't get the datetime specified.");
                              return;
                            },
                            LocalResult::Single(datetime) => datetime,
                            LocalResult::Ambiguous(_, _) => {
                              println!("Datetime given is ambiguous. Maybe try to be more clear in your time?");
                              return;
                            }
                          }
                        },
                        None => {
                          let today: Date<Local> = Local::today();
                          let today_time_option: Option<DateTime<Local>> = today.and_time(*endtime);
                          match today_time_option {
                            Some(datetime) => datetime,
                            None => {
                              println!("The date is assumed to be today, however, the time given seems to make it invalid.");
                              return;
                            }
                          }
                        }
                      };
                      
                      let duration = datetime.signed_duration_since(Local::now());
                      let minutes = duration.num_minutes() + 1; // + 1 since it undershoots, apparently
                      cold_turkey.args(["-lock", &minutes.to_string()]).spawn();
                    }
                  }
                },
                None => {
                  cold_turkey.spawn();
                }
              }
            }
          }
        },
        Command::Stop{ block_name } => {
          cold_turkey.args(["-stop", block_name]).spawn();
        },
        Command::Add{ block_name, url, except } => {
          let except_cmd: &str = if *except { "-exception" } else { "-web" };
          cold_turkey.args(["-add", block_name, except_cmd, url]).spawn();
        }
        Command::Toggle{ block_name } => {
          cold_turkey.args(["-toggle", block_name]).spawn();
        },
        Command::Suggest => println!("I suggest you go away!"),
      }
    }, None => {
      match cold_turkey.spawn() {
        Ok(_) => println!("Launches Cold Turkey!"),
        Err(_) => println!("Failed to launch Cold Turkey. Make sure you have it installed."),
      };
    }
  }
}
