#![allow(unused_must_use)]

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
  -b --break-first  Starts the pomodoro with the block name unblocked first
  -t --timer        Displays a countdown pomodoro timer
  --loops=LOOPS     Number of loops in a pomodoro session
*/

use std::process;
use chrono::{DateTime, Local, FixedOffset};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct ColdTurkey {
  #[clap(subcommand)]
  command: Option<Command>,
}

#[derive(Subcommand)]
enum ForSubcommands {
  For {
    minutes: u32
  },
  Until {
    #[clap(parse(try_from_str = DateTime::parse_from_rfc2822))]
    endtime: DateTime<FixedOffset>,
  }
}

#[derive(Subcommand)]
enum Command {
  Start {
    block_name: String,
    #[clap(short, long)]
    password: Option<String>,
    #[clap(subcommand)]
    subcommand: Option<ForSubcommands>,
  },
  Stop {
    block_name: String,
  },
  Add {
    block_name: String,
    url: String,
    #[clap(short, long)]
    except: bool,
  },
  Toggle {
    block_name: String,
  },
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
                    ForSubcommands::Until{ endtime } => {
                      // A few qualms about this:
                      // 1. The timing isn't very exact. e.g. I tried 18:00:00 and CT goes 17:59
                      // 2. Only supports RFC 2822. (yes I have to include timezone)
                      // 3. It's annoying
                      let duration = endtime.signed_duration_since(Local::now());
                      let minutes = duration.num_minutes();
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
