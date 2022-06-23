// Get rid of unused_must_use errors for now
#![allow(unused_must_use)]
use chrono::{Date, DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use clap::{Parser, Subcommand, ColorChoice};
use std::process;

pub mod convert;
pub mod schedule;
pub mod suggest;

#[derive(Parser)]
#[clap(
    name = "ctk",
    author = "Nguyen Tran (GitHub: ngtr6788)",
    version = "0.1.0",
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
    /// Interactively suggest what blocks you want Cold Turkey to have
    Suggest,
}

fn main() {
    let mut cold_turkey =
        process::Command::new(r"C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe");
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
                    Some(p) => {
                        match cold_turkey.args(["-password", p]).spawn() {
                            Ok(_) => {
                                println!("Starts blocking {} with a password", block_name);
                                println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                            }
                            Err(_) => {
                                println!("Cannot run the command: ctk start --password. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                            }
                        };
                    }
                    None => match subcommand {
                        Some(method) => match method {
                            ForSubcommands::For { minutes } => {
                                match cold_turkey.args(["-lock", &minutes.to_string()]).spawn() {
                                    Ok(_) => {
                                        println!(
                                            "Starts blocking {} locked for {} minutes",
                                            block_name, minutes
                                        );
                                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                                    }
                                    Err(_) => {
                                        println!("Cannot run the command: ctk start for. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
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
                                                println!("Can't get the datetime specified.");
                                                return;
                                            }
                                            LocalResult::Single(datetime) => datetime,
                                            LocalResult::Ambiguous(_, _) => {
                                                println!("Datetime given is ambiguous. Maybe try to be more clear in your time?");
                                                return;
                                            }
                                        }
                                    }
                                    None => {
                                        let today: Date<Local> = Local::today();
                                        let today_time_option: Option<DateTime<Local>> =
                                            today.and_time(*endtime);
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
                                // If duration is exactly a multiple of 60, do not round up
                                let duration_minutes = if duration.num_seconds() % 60 == 0 {
                                    duration.num_minutes()
                                } else {
                                    duration.num_minutes() + 1
                                };
                                match cold_turkey.args(["-lock", &duration_minutes.to_string()]).spawn() {
                                    Ok(_) => {
                                        println!(
                                            "Starts blocking {} locked until {}",
                                            block_name,
                                            datetime.to_rfc2822()
                                        );
                                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                                    }
                                    Err(_) => {
                                        println!("Cannot run the command: ctk start until. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                                    }
                                };
                            }
                        },
                        None => {
                            match cold_turkey.spawn() {
                                Ok(_) => {
                                    println!("Starts blocking {}", block_name);
                                    println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                                }
                                Err(_) => {
                                    println!("Cannot run the command: ctk start. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                                }
                            };
                        }
                    },
                }
            }
            Command::Stop { block_name } => {
                match cold_turkey.args(["-stop", block_name]).spawn() {
                    Ok(_) => {
                        println!("Stops blocking {}", block_name);
                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                    }
                    Err(_) => {
                        println!("Cannot run the command: ctk stop. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
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
                        println!("Adds url {} to block {}", url, block_name);
                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                    }
                    Err(_) => {
                        println!("Cannot run the command: ctk add. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                    }
                }
            }
            Command::Toggle { block_name } => {
                match cold_turkey.args(["-toggle", block_name]).spawn() {
                    Ok(_) => {
                        println!("Toggles block {}", block_name);
                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                    }
                    Err(_) => {
                        println!("Cannot run the command: ctk toggle. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                    }
                };
            }
            Command::Suggest => {
                suggest::suggest();
            }
        },
        None => {
            match cold_turkey.spawn() {
                Ok(_) => println!("Launches Cold Turkey!"),
                Err(_) => {
                    println!(
                        r"Looks like you don't have Cold Turkey installed on C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe"
                    );
                    println!("If you do have it installed, please put Cold Turkey Blocker.exe in the folder mentioned.");
                    println!("If not, you're welcome to download it at getcoldturkey.com.");
                }
            };
        }
    }
}
