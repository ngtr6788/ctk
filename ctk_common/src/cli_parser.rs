use chrono::{NaiveDate, NaiveTime};
use clap::{ColorChoice, Parser, Subcommand};
use crate::{convert::*, ctsettings::get_ct_settings};

fn get_all_ct_blocks() -> Vec<String> {
  let ct_settings = get_ct_settings();
  ct_settings.map_or(Vec::new(), |settings| settings.block_list_info.blocks.into_keys().collect())
}

#[derive(Parser)]
#[command(
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
pub struct ColdTurkey {
  #[command(subcommand)]
  pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum StartSubcommands {
  /// Set a time period to block
  For {
    /// How long to block in minutes
    #[arg(long)]
    minutes: Option<u32>,
    #[arg(long)]
    hours: Option<u32>,
    #[arg(long)]
    days: Option<u32>,
  },
  /// Set when the block is finished
  Until {
    #[arg(value_parser = str_to_time)]
    /// The time of the end of a block
    endtime: NaiveTime,
    #[arg(value_parser = str_to_date)]
    /// The date of the end of a block. Defaults to today if not given
    enddate: Option<NaiveDate>,
  },
}

#[derive(Subcommand)]
pub enum Command {
  /// Start a block
  Start {
    /// The name of the Cold Turkey block
    #[arg(value_parser = get_all_ct_blocks())]
    block_name: String,
    #[arg(short, long)]
    /// Password to lock the block
    password: bool,
    #[command(subcommand)]
    subcommand: Option<StartSubcommands>,
  },
  /// Stop a block
  Stop {
    /// The name of the Cold Turkey block
    #[arg(value_parser = get_all_ct_blocks())]
    block_name: String,
  },
  /// Add websites (urls) to a block
  Add {
    /// The name of the Cold Turkey block
    #[arg(value_parser = get_all_ct_blocks())]
    block_name: String,
    /// The url to add in the block
    url: String,
    #[arg(short, long)]
    /// Whether it is black or white-listed
    except: bool,
  },
  /// Turn on if off, turn off if on
  Toggle {
    /// The name of the Cold Turkey block
    #[arg(value_parser = get_all_ct_blocks())]
    block_name: String,
  },
  /// Interactively suggest what blocks you want Cold Turkey to have
  Suggest,
  /// List all the blocks in alphabetical order by default
  List,
  /// Installs Cold Turkey
  Install {
    /// Force installing Cold Turkey, regardless if Cold Turkey Blocker exists
    #[arg(short, long)]
    force: bool,
  },
}

