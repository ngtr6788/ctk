#![allow(dead_code, unused_must_use, unused_imports, unused_variables)]
use chrono::{
    Date, DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, ParseResult, TimeZone,
};
use clap::{Parser, Subcommand};
use rand::Rng;
use rpassword;
use serde::{Deserialize, Serialize};
use shlex;
use std::collections::HashMap;
use std::env;
use std::io;
use std::process;

/**
Usage:
    suggest new <block_name>
    suggest remove <block_name>
    suggest unlock <block_name>
    suggest lock <block_name> (random | range | restart | password)
    suggest config <block_name> random <length> [-l]
    suggest config <block_name> range <start_time> <end_time> [-ul]
    suggest config <block_name> restart [-ul]
    suggest config <block_name> password [-l]
    suggest nobreak <block_name>
    suggest pomodoro <block_name> <block_minutes> <break_minutes>
    suggest allowance <block_name> <minutes>
    suggest (add | delete) <block_name> web <url> [-e]
    suggest (add | delete) <block_name> (file | folder | win10 | title) <path>
    suggest settings <block_name>
    suggest blocks [-v]
    suggest save [<file_name>]
    suggest pwd
    suggest (q | quit)
Options:
    -u --unlocked   Block is unlocked after it's done
    -l --lock       Simultaneously locks with that type and configures it
    -v --verbose    Displays all blocks as well as each block's settings
    -e --except     Adds a URL as an exception
*/

#[derive(Parser)]
#[clap(
    name = "ctk",
    author = "Nguyen Tran (GitHub: ngtr6788)",
    version = "0.0.1"
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
        minutes: u32,
    },
    /// Set when the block is finished
    Until {
        #[clap(parse(try_from_str = str_to_time))]
        /// The time of the end of a block
        endtime: NaiveTime,
        #[clap(parse(try_from_str = str_to_date))]
        /// The date of the end of a block. Defaults to today if not given
        enddate: Option<NaiveDate>,
    },
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
    const ALLOWED_PARSE: [&str; 5] = [
        "%d %B %Y",
        "%e %B %Y",
        "%B %d %Y, %B %e %Y",
        "%F",
        "%d/%m/%Y",
    ];
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
    Suggest,
}

#[derive(Parser, Debug)]
enum Suggest {
    NewBlock {
        block_name: String,
    },
    RemoveBlock {
        block_name: String,
    },
    Unlock {
        block_name: String,
    },
    Lock {
        block_name: String,
        #[clap(subcommand)]
        lock_method: LockMethod,
    },
    Config {
        block_name: String,
        #[clap(subcommand)]
        lock_method: LockMethodConfig,
        #[clap(short, long)]
        lock: bool,
    },
    NoBreak {
        block_name: String,
    },
    Allowance {
        block_name: String,
        allowance_minutes: u8,
    },
    Pomodoro {
        block_name: String,
        lock_minutes: u8,
        break_minutes: u8,
    },
    Add {
        block_name: String,
        #[clap(subcommand)]
        path_type: PathType,
        path: String,
    },
    Delete {
        block_name: String,
        #[clap(subcommand)]
        path_type: PathType,
        path: String,
    },
    Settings {
        block_name: String,
    },
    List {
        #[clap(short, long)]
        verbose: bool,
    },
    Save {
        file_name: Option<String>,
    },
    Pwd,
    Quit,
}

#[derive(Clone, Copy, Subcommand, Debug)]
enum LockMethod {
    Random,
    Range,
    Restart,
    Password,
}

#[derive(Subcommand, Debug)]
enum LockMethodConfig {
    Random {
        length: u8,
    },
    Range {
        start_time: NaiveTime,
        end_time: NaiveTime,
        #[clap(short, long)]
        unlocked: bool,
    },
    Restart {
        #[clap(short, long)]
        unlocked: bool,
    },
    Password,
}

#[derive(Subcommand, Debug)]
enum PathType {
    Web {
        #[clap(short, long)]
        except: bool,
    },
    File,
    Folder,
    Win10,
    Title,
}

#[derive(Debug)]
struct BlockSettings {
    enabled: bool,
    lock: Option<LockMethod>,
    lock_unblock: bool,
    restart_unblock: bool,
    password: String,
    random_text_length: u8,
    break_type: BreakType,
    window: Window,
    users: String,
    web: Vec<String>,
    exceptions: Vec<String>,
    apps: Vec<String>,
    schedule: Vec<String>,
    custom_users: Vec<String>,
}

#[derive(Debug)]
enum BreakType {
    None,
    Allowance { minutes: u8 },
    Pomodoro { block_min: u8, break_min: u8 },
}

#[derive(Debug)]
struct Window {
    lock: bool,
    start_time: NaiveTime,
    end_time: NaiveTime,
}

impl BlockSettings {
    fn new() -> Self {
        let new_settings: BlockSettings = BlockSettings {
            enabled: false,
            lock: None,
            lock_unblock: true,
            restart_unblock: true,
            password: String::new(),
            random_text_length: 30,
            break_type: BreakType::None,
            window: Window {
                lock: true,
                start_time: NaiveTime::from_hms(9, 0, 0),
                end_time: NaiveTime::from_hms(17, 0, 0),
            },
            users: String::new(),
            web: Vec::new(),
            exceptions: Vec::new(),
            apps: Vec::new(),
            schedule: Vec::new(),
            custom_users: Vec::new(),
        };
        return new_settings;
    }
}

/**
 * stdin_to_suggest reads input from stdin, treats them like command line
 * arguments and returns a Suggest enum parsed by clap
 */
fn stdin_to_suggest() -> Suggest {
    loop {
        let mut suggest_input: String = String::new();
        match io::stdin().read_line(&mut suggest_input) {
            Ok(_) => {
                let shlex_parse: Option<Vec<String>> = shlex::split(&suggest_input);
                match shlex_parse {
                    Some(mut cmd_input) => {
                        // For Windows, there is a carriage return at the very end,
                        // so this should get rid of it
                        if let Some(last) = cmd_input.last_mut() {
                            *last = last.trim().to_string();
                        };

                        cmd_input.insert(0, "suggest".to_string());
                        match Suggest::try_parse_from(cmd_input.into_iter()) {
                            Ok(suggest_cmd) => {
                                return suggest_cmd;
                            }
                            Err(clap_error) => {
                                clap_error.print();
                                continue;
                            }
                        }
                    }
                    None => {
                        println!("Can't parse this string: pleasy try again.");
                        continue;
                    }
                }
            }
            Err(_) => {
                println!("Can't read any input: please try again.");
                continue;
            }
        }
    }
}

fn suggest() {
    let mut list_of_blocks: HashMap<String, BlockSettings> = HashMap::new();

    loop {
        // This section creates the suggest_cmd enum struct thing from
        // reading from stdin and parsed it with clap.
        let suggest_cmd: Suggest = stdin_to_suggest();

        match suggest_cmd {
            Suggest::NewBlock { block_name } => {
                if list_of_blocks.contains_key(&block_name) {
                    println!("Block {} already exists", &block_name);
                } else {
                    println!("Block {} added", &block_name);
                    list_of_blocks.insert(block_name, BlockSettings::new());
                }
            }
            Suggest::RemoveBlock { block_name } => {
                if list_of_blocks.contains_key(&block_name) {
                    println!("Block {} removed", &block_name);
                    list_of_blocks.remove(&block_name);
                } else {
                    println!("Block {} does not exist", &block_name);
                }
            }
            Suggest::Unlock { block_name } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.lock = None;
                    println!("Block {} unlocked", &block_name);
                }
                None => println!("Block {} does not exist", &block_name),
            },
            Suggest::Lock {
                block_name,
                lock_method,
            } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.lock = Some(lock_method);
                    println!("Block {} has been locked by {:?}", block_name, lock_method);
                }
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::Config {
                block_name,
                lock_method,
                lock,
            } => {
                let is_locked = if lock { " and locked" } else { "" };
                match list_of_blocks.get_mut(&block_name) {
                    Some(bs) => match lock_method {
                        LockMethodConfig::Random { length } => {
                            bs.random_text_length = length;
                            if lock {
                                bs.lock = Some(LockMethod::Random);
                            }
                            println!(
                                "Block {} was configured{} with {} random characters",
                                block_name, is_locked, length
                            );
                        }
                        LockMethodConfig::Range {
                            start_time,
                            end_time,
                            unlocked,
                        } => {
                            bs.window = Window {
                                lock: !unlocked,
                                start_time,
                                end_time,
                            };
                            if lock {
                                bs.lock = Some(LockMethod::Range);
                            }
                            println!(
                                "Block {} was configured{} with a time range",
                                block_name, is_locked
                            );
                        }
                        LockMethodConfig::Restart { unlocked } => {
                            bs.restart_unblock = unlocked;
                            if lock {
                                bs.lock = Some(LockMethod::Restart);
                            }
                            println!(
                                "Block {} was configured{} by restart",
                                block_name, is_locked
                            );
                        }
                        LockMethodConfig::Password => {
                            if let Ok(password) =
                                rpassword::prompt_password("Please enter your password: ")
                            {
                                bs.password = password;
                            }
                            if lock {
                                bs.lock = Some(LockMethod::Password);
                            }
                            println!(
                                "Block {} was configured{} with a password",
                                block_name, is_locked
                            );
                        }
                    },
                    None => println!("Block {} does not exist", block_name),
                }
            }
            Suggest::NoBreak { block_name } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.break_type = BreakType::None;
                    println!("Blocks {} with no breaks", block_name)
                }
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::Allowance {
                block_name,
                allowance_minutes,
            } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.break_type = BreakType::Allowance {
                        minutes: allowance_minutes,
                    };
                    println!(
                        "Block {} has an allowance of {} min",
                        block_name, allowance_minutes
                    );
                }
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::Pomodoro {
                block_name,
                lock_minutes,
                break_minutes,
            } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.break_type = BreakType::Pomodoro {
                        block_min: lock_minutes,
                        break_min: break_minutes,
                    };
                    println!(
                        "Block {} has pomodoro {} block min, {} break min",
                        block_name, lock_minutes, break_minutes
                    )
                }
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::Add {
                block_name,
                path_type,
                path,
            } => println!("Added {} of {:?} to {}", path, path_type, block_name),
            Suggest::Delete {
                block_name,
                path_type,
                path,
            } => println!("Deleted {} of {:?} from {}", path, path_type, block_name),
            Suggest::Settings { block_name } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => println!("{:?}", bs),
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::List { verbose } => {
                if verbose {
                    println!("{:?}", &list_of_blocks);
                } else {
                    for key in list_of_blocks.keys() {
                        println!("{}", key);
                    }
                }
            }
            Suggest::Save { file_name } => match file_name {
                Some(name) => println!("Saved to {}.ctbbl", name),
                None => {
                    let num: u64 = rand::thread_rng().gen();
                    println!("Saved to ctk_{}.ctbbl", num);
                }
            },
            Suggest::Pwd => {
                if let Ok(current_dir) = env::current_dir() {
                    println!("{}", current_dir.display());
                }
            }
            Suggest::Quit => break,
        }
    }
}

fn main() {
    let mut cold_turkey =
        process::Command::new(r"C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe");
    let args = ColdTurkey::parse();
    match &args.command {
        Some(cmd) => {
            match &cmd {
                Command::Start {
                    block_name,
                    password,
                    subcommand,
                } => {
                    cold_turkey.args(["-start", block_name]);
                    match password {
                        Some(p) => {
                            cold_turkey.args(["-password", p]).spawn();
                        }
                        None => {
                            match subcommand {
                                Some(method) => {
                                    match method {
                                        ForSubcommands::For { minutes } => {
                                            cold_turkey
                                                .args(["-lock", &minutes.to_string()])
                                                .spawn();
                                        }
                                        ForSubcommands::Until { endtime, enddate } => {
                                            let datetime: DateTime<Local> = match enddate {
                                                Some(date) => {
                                                    let naive_datetime: NaiveDateTime =
                                                        date.and_time(*endtime);
                                                    let datetime_result: LocalResult<
                                                        DateTime<Local>,
                                                    > = Local.from_local_datetime(&naive_datetime);
                                                    match datetime_result {
                                                        LocalResult::None => {
                                                            println!(
                                                                "Can't get the datetime specified."
                                                            );
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

                                            let duration =
                                                datetime.signed_duration_since(Local::now());
                                            let minutes = duration.num_minutes() + 1; // + 1 since it undershoots, apparently
                                            cold_turkey
                                                .args(["-lock", &minutes.to_string()])
                                                .spawn();
                                        }
                                    }
                                }
                                None => {
                                    cold_turkey.spawn();
                                }
                            }
                        }
                    }
                }
                Command::Stop { block_name } => {
                    cold_turkey.args(["-stop", block_name]).spawn();
                }
                Command::Add {
                    block_name,
                    url,
                    except,
                } => {
                    let except_cmd: &str = if *except { "-exception" } else { "-web" };
                    cold_turkey
                        .args(["-add", block_name, except_cmd, url])
                        .spawn();
                }
                Command::Toggle { block_name } => {
                    cold_turkey.args(["-toggle", block_name]).spawn();
                }
                Command::Suggest => {
                    suggest();
                }
            }
        }
        None => {
            match cold_turkey.spawn() {
                Ok(_) => println!("Launches Cold Turkey!"),
                Err(_) => {
                    println!("Failed to launch Cold Turkey. Make sure you have it installed.")
                }
            };
        }
    }
}
