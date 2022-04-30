use chrono::{
    Date, DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, ParseResult, Timelike, TimeZone
};
use clap::{Parser, Subcommand};
use rand::Rng;
use rpassword;
use serde::Serialize;
use serde_json;
use shlex;
use std::collections::HashMap;
use std::env;
use std::io;
use std::io::Write;
use std::process;
use std::path::Path;
use std::fs::File;

#[derive(Parser)]
#[clap(
    name = "ctk",
    author = "Nguyen Tran (GitHub: ngtr6788)",
    version = "0.1.0"
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
    const ALLOWED_PARSE: [&str; 6] = [
        "%d %B %Y",
        "%e %B %Y",
        "%B %d %Y", 
        "%B %e %Y",
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
    /// Interactively suggest what blocks you want Cold Turkey to have
    Suggest,
}

#[derive(Parser, Debug)]
/// Suggests what blocks Cold Turkey should have
enum Suggest {
    /// Creates a new block
    NewBlock {
        /// Name your wishlist Cold Turkey block
        block_name: String,
    },
    /// Removes a block
    RemoveBlock {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
    },
    /// Unlocks a block (no lock type is set)
    Unlock {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
    },
    /// Locks a block with various methods
    Lock {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
        #[clap(subcommand)]
        /// How a block is locked when block is turned on
        lock_method: LockMethod,
    },
    /// Configures a block-locking method with custom settings
    Config {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
        #[clap(subcommand)]
        /// How a block is locked when block is turned on
        lock_method: LockMethodConfig,
        #[clap(short, long)]
        /// Locks this block with this method
        lock: bool,
    },
    /// When set, blocks without breaks
    Nobreak {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
    },
    /// Allows unblocked until time is up
    Allowance {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
        /// How long to allow unblocked
        allowance_minutes: u16,
    },
    /// Blocks for a certain time, then breaks for a certain time
    Pomodoro {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
        /// How long for the block to be blocked
        lock_minutes: u16,
        /// How long for the block to relax its block
        break_minutes: u16,
    },
    /// Adds a website or application to a block
    Add {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
        #[clap(subcommand)]
        /// What the path should be considered as
        path_type: PathType,
        /// File path on a computer or URL of a website
        path: String,
    },
    /// Deletes a website or application from a block
    Delete {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
        #[clap(subcommand)]
        /// What the path should be considered as
        path_type: PathType,
        /// File path on a computer or URL of a website
        path: String,
    },
    /// Makes a block unscheduled and continuous
    Continuous {
        block_name: String,
    },
    /// Schedules a block's blocking time over a week
    Schedule {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String
    },
    /// Shows all the settings of a block
    Settings {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
    },
    /// Lists all the blocks (verbose optional)
    List {
        #[clap(short, long)]
        /// Displays all the blocks as well as their settings
        verbose: bool,
    },
    /// Saves all the block settings to a .ctbbl file in pretty JSON format
    Save {
        /// To be saved as [file_name].ctbbl
        file_name: Option<String>,
    },
    /// Shows current directory
    Pwd,
    /// Quits suggest
    Quit,
}

#[derive(Clone, Copy, Subcommand, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum LockMethod {
    /// No lock at all
    None,
    /// Locks with a random string
    Random,
    #[serde(rename = "window")]
    /// Locks/Unlocks within a time range
    Range,
    /// Locks until computer restart
    Restart,
    /// Locks with a password
    Password,
}

#[derive(Subcommand, Debug)]
enum LockMethodConfig {
    /// Locks with a random string
    Random {
        /// Length of the random string
        length: u16,
    },
    /// Locks/Unlocks within a time range
    Range {
        #[clap(parse(try_from_str = str_to_time))]
        /// Start of block range
        start_time: NaiveTime,
        #[clap(parse(try_from_str = str_to_time))]
        /// End of block range
        end_time: NaiveTime,
        #[clap(short, long)]
        /// Set the block to be unlocked in this range instead of locked
        unlocked: bool,
    },
    /// Locks until computer restart
    Restart {
        #[clap(short, long)]
        /// Set the block to be unlocked after computer restart
        unlocked: bool,
    },
    /// Locks with a password
    Password,
}

#[derive(Subcommand, Debug)]
enum PathType {
    /// Website path
    Web {
        #[clap(short, long)]
        /// Adds this URL as an exception
        except: bool,
    },
    /// Applications in a file (e.g. .exe files in Windows)
    File,
    /// Many applications in a folder
    Folder,
    /// Windows 10 application
    Win10,
    /// Matches any window title with a name
    Title,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct BlockSettings {
    #[serde(rename = "type")]
    sched_type: SchedType,
    lock: LockMethod,
    lock_unblock: String,
    restart_unblock: String,
    password: String,
    random_text_length: String,
    #[serde(rename = "break")]
    break_type: String,
    window: String,
    users: String,
    web: Vec<String>,
    exceptions: Vec<String>,
    apps: Vec<String>,
    schedule: Vec<ScheduleBlock>,
    custom_users: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
enum SchedType {
    Continuous,
    Scheduled
}

#[derive(Debug, Serialize)]
struct Window {
    lock: bool,
    start_time: String,
    end_time: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct ScheduleBlock {
    id: usize,
    start_time: String,
    end_time: String,
    #[serde(rename = "break")]
    break_type: String,
}

impl BlockSettings {
    fn new() -> Self {
        let new_settings: BlockSettings = BlockSettings {
            sched_type: SchedType::Continuous,
            lock: LockMethod::None,
            lock_unblock: true.to_string(),
            restart_unblock: true.to_string(),
            password: String::new(),
            random_text_length: 30.to_string(),
            break_type: "none".to_owned(),
            window: "lock@9,0@17,0".to_string(),
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
        print!("> suggest ");
        io::stdout().flush();
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
                        println!("Can't parse this command: pleasy try again.");
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
                    bs.lock = LockMethod::None;
                    println!("Block {} unlocked", &block_name);
                }
                None => println!("Block {} does not exist", &block_name),
            },
            Suggest::Lock {
                block_name,
                lock_method,
            } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.lock = lock_method;
                    match lock_method {
                        LockMethod::None => println!("Block {} unlocked", block_name),
                        LockMethod::Random => println!("Block {} locked by a string of random characters", block_name),
                        LockMethod::Range => println!("Block {} locked within some time range", block_name),
                        LockMethod::Restart => println!("Block {} locked until restart", block_name),
                        LockMethod::Password => println!("Block {} locked with a password", block_name),
                    };
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
                            bs.random_text_length = length.to_string();
                            if lock {
                                bs.lock = LockMethod::Random;
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
                            let window_string = if unlocked { "unlock".to_owned() } else { "lock".to_owned() };
                            let start_string = "@".to_owned() + &start_time.hour().to_string() + "," + &start_time.minute().to_string();
                            let end_string = "@".to_owned() + &end_time.hour().to_string() + "," + &end_time.minute().to_string();
                            bs.window = window_string + &start_string + &end_string;
                            if lock {
                                bs.lock = LockMethod::Range;
                            }
                            println!(
                                "Block {} was configured{} with a time range",
                                block_name, is_locked
                            );
                        }
                        LockMethodConfig::Restart { unlocked } => {
                            bs.restart_unblock = unlocked.to_string();
                            if lock {
                                bs.lock = LockMethod::Restart;
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
                                bs.lock = LockMethod::Password;
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
            Suggest::Nobreak { block_name } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.break_type = "none".to_owned();
                    println!("Blocks {} with no breaks", block_name)
                }
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::Allowance {
                block_name,
                allowance_minutes,
            } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.break_type = allowance_minutes.to_string();
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
                    bs.break_type = lock_minutes.to_string() + "," + &break_minutes.to_string();
                    println!(
                        "Block {} has pomodoro of {} block min, {} break min",
                        block_name, lock_minutes, break_minutes
                    )
                }
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::Add {
                block_name,
                path_type,
                mut path,
            } => {
                path = path.replace("\\", "/");
                match list_of_blocks.get_mut(&block_name) {
                    Some(bs) => match path_type {
                        PathType::Web { except } => {
                            if except {
                                println!("Added {} to {} as a website exception", &path, block_name);
                                bs.exceptions.push(path);
                            } else {
                                println!("Added {} to {} as a website", &path, block_name);
                                bs.web.push(path);
                            }
                        }
                        PathType::File => {
                            println!("Added {} to {} as a file", &path, block_name);
                            let app = "file:".to_owned() + &path;
                            bs.apps.push(app);
                        }
                        PathType::Folder => {
                            println!("Added {} to {} as a folder", &path, block_name);
                            let app = "app:".to_owned() + &path;
                            bs.apps.push(app);
                        }
                        PathType::Win10 => {
                            println!("Added {} to {} as a Windows 10 application", &path, block_name);
                            let app = "win10:".to_owned() + &path;
                            bs.apps.push(app);
                        }
                        PathType::Title => {
                            println!("Added {} to {} as a window title", &path, block_name);
                            let app = "title:".to_owned() + &path;
                            bs.apps.push(app);
                        }
                    },
                    None => println!("Block {} does not exist", block_name),
                }
            }
            Suggest::Delete {
                block_name,
                path_type,
                path,
            } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => match path_type {
                    PathType::Web { except } => {
                        let remove_vec: &mut Vec<String> = if except {
                            &mut bs.exceptions
                        } else {
                            &mut bs.web
                        };
                        if let Some(idx) = remove_vec.iter().position(|s| *s == path) {
                            remove_vec.swap_remove(idx);
                            println!("Web path {} removed from block {}", &path, &block_name);
                        } else {
                            println!("Web path {} does not exist in block {}", &path, &block_name);
                        }
                    }
                    PathType::File => {
                        let app = "file:".to_owned() + &path;
                        if let Some(idx) = bs.apps.iter().position(|a| *a == app) {
                            bs.apps.swap_remove(idx);
                            println!("File {} removed from {}", &path, &block_name);
                        } else {
                            println!("File {} does not exist in {}", &path, &block_name);
                        }
                    }
                    PathType::Folder => {
                        let app = "folder:".to_owned() + &path;
                        if let Some(idx) = bs.apps.iter().position(|a| *a == app) {
                            bs.apps.swap_remove(idx);
                            println!("Folder {} removed from {}", &path, &block_name);
                        } else {
                            println!("Folder {} does not exist in {}", &path, &block_name);
                        }
                    }
                    PathType::Title => {
                        let app = "title:".to_owned() + &path;
                        if let Some(idx) = bs.apps.iter().position(|a| *a == app) {
                            bs.apps.swap_remove(idx);
                            println!("Window title {} removed from {}", &path, &block_name);
                        } else {
                            println!("Window title {} does not exist in {}", &path, &block_name);
                        }
                    }
                    PathType::Win10 => {
                        let app = "win10:".to_owned() + &path;
                        if let Some(idx) = bs.apps.iter().position(|a| *a == app) {
                            bs.apps.swap_remove(idx);
                            println!("Windows 10 application {} removed from {}", &path, &block_name);
                        } else {
                            println!("Windows 10 application {} does not exist in {}", &path, &block_name);
                        }
                    }
                },
                None => println!("Block {} does not exist", &block_name),
            },
            Suggest::Continuous { block_name } => {
                match list_of_blocks.get_mut(&block_name) {
                    Some(bs) => {
                        bs.sched_type = SchedType::Continuous;
                        println!("Made block {} to be blocked continously without schedule", &block_name);
                    },
                    None => println!("Block {} does not exist", block_name),
                }
            },
            Suggest::Schedule { block_name } => {
                match list_of_blocks.get_mut(&block_name) {
                    Some(bs) => {
                        bs.sched_type = SchedType::Scheduled;
                        bs.schedule = schedule(&block_name);
                        println!("Added a schedule to block {}", &block_name);
                    },
                    None => println!("Block {} does not exist", block_name),
                }
            },
            Suggest::Settings { block_name } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => println!("{:?}", bs),
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::List { verbose } => {
                if verbose {
                    if let Ok(pretty_json) = serde_json::to_string_pretty(&list_of_blocks) {
                        println!("{}", pretty_json);
                    } else {
                        println!("Due to unexpected reasons, we cannot pretty display the blocks with all its settings");
                    }
                } else {
                    for key in list_of_blocks.keys() {
                        println!("{}", key);
                    }
                }
            }
            Suggest::Save { file_name } => {
                let final_file: String = match file_name {
                    Some(name) => name + ".ctbbl",
                    None => {
                        let num: u64 = rand::thread_rng().gen();
                        "ctk_".to_owned() + &num.to_string() + ".ctbbl"
                    }
                };
                
                let path = Path::new(&final_file);
                let display = path.display();
                
                match File::create(&path) {
                    Ok(file) => {
                        match serde_json::to_writer_pretty(file, &list_of_blocks) {
                            Ok(_) => println!("Successfully saved to {} in current directory", display),
                            Err(why) => println!("Could not write to {}: {}", display, why),
                        }
                    }
                    Err(why) => {
                        println!("Could not create {}: {}", display, why);
                    }
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

#[derive(Parser)]
enum Schedule {
    Add {
        #[clap(parse(try_from_str = str_to_time))]
        start_time: NaiveTime,
        #[clap(parse(try_from_str = str_to_time))]
        end_time: NaiveTime,
        #[clap(long)]
        sun: bool,
        #[clap(long)]
        mon: bool,
        #[clap(long)]
        tue: bool,
        #[clap(long)]
        wed: bool,
        #[clap(long)]
        thu: bool,
        #[clap(long)]
        fri: bool,
        #[clap(long)]
        sat: bool,
        #[clap(long)]
        wkday: bool,
        #[clap(long)]
        wkend: bool,
        #[clap(short, long)]
        all: bool,
        #[clap(subcommand)]
        break_type: ScheduleBreak,
    },
    Edit {
        #[clap(long)]
        id: usize,
        #[clap(parse(try_from_str = str_to_day))]
        day: Day,
        #[clap(parse(try_from_str = str_to_time))]
        start_time: NaiveTime,
        #[clap(parse(try_from_str = str_to_time))]
        end_time: NaiveTime,
        #[clap(subcommand)]
        break_type: ScheduleBreak,
    },
    Remove {
        ids: Vec<usize>,
        #[clap(long)]
        all: bool,
    },
    Print,
    Done,
}

#[derive(Subcommand)]
enum Day {
    Sun,
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat
}

fn str_to_day<'a, 'b>(s: &'a str) -> Result<Day, &'b str> {
    match s {
        "sun" => Ok(Day::Sun),
        "mon" => Ok(Day::Mon),
        "tue" => Ok(Day::Tue),
        "wed" => Ok(Day::Wed),
        "thu" => Ok(Day::Thu),
        "fri" => Ok(Day::Fri),
        "sat" => Ok(Day::Sat),
        _ => Err("Not a valid day of the week. Must be sun, mon, tue, wed, thu, fri, sat")
    }
}

#[derive(Subcommand)]
enum ScheduleBreak {
    /// When set, blocks without breaks
    Nobreak,
    /// Allows unblocked until time is up
    Allowance {
        /// How long to allow unblocked
        allowance_minutes: u16,
    },
    /// Blocks for a certain time, then breaks for a certain time
    Pomodoro {
        /// How long for the block to be blocked
        lock_minutes: u16,
        /// How long for the block to relax its block
        break_minutes: u16,
    },
}

fn stdin_to_schedule(block_name: &str) -> Schedule {
    loop {
        print!(">> schedule [{}] ", &block_name);
        io::stdout().flush();
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

                        cmd_input.insert(0, "schedule".to_string());
                        match Schedule::try_parse_from(cmd_input.into_iter()) {
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
                        println!("Can't parse this command: pleasy try again.");
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

fn schedule(block_name: &str) -> Vec<ScheduleBlock> {
    let mut final_schedule: Vec<ScheduleBlock> = Vec::new();

    loop {
        let schedule_cmd: Schedule = stdin_to_schedule(&block_name);

        match schedule_cmd {
            Schedule::Add { 
                start_time, 
                end_time, 
                mut sun, 
                mut mon, 
                mut tue,
                mut wed,
                mut thu,
                mut fri,
                mut sat,
                wkday,
                wkend,
                all,
                break_type,     
            } => {
                let start_string_end = ",".to_owned() + &start_time.hour().to_string() + "," + &start_time.minute().to_string();
                let end_string_end = ",".to_owned() + &end_time.hour().to_string() + "," + &end_time.minute().to_string();

                let break_string = match break_type {
                    ScheduleBreak::Nobreak => "none".to_string(),
                    ScheduleBreak::Allowance { allowance_minutes } => allowance_minutes.to_string(),
                    ScheduleBreak::Pomodoro { lock_minutes, break_minutes } => lock_minutes.to_string() + "," + &break_minutes.to_string(),
                };

                if all {
                    sun = true;
                    mon = true;
                    tue = true;
                    wed = true;
                    thu = true;
                    fri = true;
                    sat = true;
                }

                if wkday {
                    mon = true;
                    tue = true;
                    wed = true;
                    thu = true;
                    fri = true;
                }

                if wkend {
                    sun = true;
                    sat = true;
                }

                const NUM_OF_DAYS_IN_WEEK: usize = 7;
                let days_of_week: [bool; NUM_OF_DAYS_IN_WEEK] = [sun, mon, tue, wed, thu, fri, sat];
                for i in 0..NUM_OF_DAYS_IN_WEEK {
                    if days_of_week[i] {
                        let start_string = i.to_string() + &start_string_end;
                        let end_string = i.to_string() + &end_string_end;

                        let block = ScheduleBlock {
                            id: final_schedule.len(),
                            start_time: start_string,
                            end_time: end_string,
                            break_type: break_string.clone(),
                        };
    
                        println!("Created a schedule block of index {}", final_schedule.len());
                        final_schedule.push(block);
                    }
                }
            },
            Schedule::Edit {
                id,
                day,
                start_time,
                end_time,
                break_type,
            } => {
                let start_string_end = ",".to_owned() + &start_time.hour().to_string() + "," + &start_time.minute().to_string();
                let end_string_end = ",".to_owned() + &end_time.hour().to_string() + "," + &end_time.minute().to_string();

                let break_string = match break_type {
                    ScheduleBreak::Nobreak => "none".to_string(),
                    ScheduleBreak::Allowance { allowance_minutes } => allowance_minutes.to_string(),
                    ScheduleBreak::Pomodoro { lock_minutes, break_minutes } => lock_minutes.to_string() + "," + &break_minutes.to_string(),
                };

                let day_num = match day {
                    Day::Sun => "0",
                    Day::Mon => "1",
                    Day::Tue => "2",
                    Day::Wed => "3",
                    Day::Thu => "4",
                    Day::Fri => "5",
                    Day::Sat => "6",
                };

                let block = ScheduleBlock {
                    id: id,
                    start_time: day_num.to_string() + &start_string_end,
                    end_time: day_num.to_string() + &end_string_end,
                    break_type: break_string.clone(),
                };

                final_schedule[id] = block;
                println!("Edited schedule block {}", id);
            }
            Schedule::Remove { ids, all } => {
                if all {
                    final_schedule.clear();
                    println!("Deleted all schedule blocks");
                } else {
                    let mut remove_element: Vec<bool> = vec![true; final_schedule.len()];
                    for i in ids {
                        println!("Deleted schedule blocks {}", i);
                        remove_element[i] = false;
                    }
                    let mut iter = remove_element.iter();
                    final_schedule.retain(|_| *iter.next().unwrap());
                    for i in 0..final_schedule.len() {
                        final_schedule[i].id = i;
                    }
                }
            },
            Schedule::Print => {
                match serde_json::to_writer_pretty(io::stdout(), &final_schedule) {
                    Ok(_) => {},
                    Err(_) => print!("Could not print to stdout"),
                }
                print!("\n");
            }
            Schedule::Done => {
                for i in 0..final_schedule.len() {
                    final_schedule[i].id = i;
                }
                println!("Done with scheduling");
                break;
            }
        }
    }

    return final_schedule;
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
                            },
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
                                        println!("Starts blocking {} locked for {} minutes", block_name, minutes);
                                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                                    },
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
                                let minutes = f64::round(duration.num_seconds() as f64 / 60.0) as i64;
                                match cold_turkey.args(["-lock", &minutes.to_string()]).spawn() {
                                    Ok(_) => {
                                        println!("Starts blocking {} locked until {}", block_name, datetime.to_rfc2822());
                                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                                    },
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
                                },
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
                    },
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
                match cold_turkey.args(["-add", block_name, except_cmd, url]).spawn() {
                    Ok(_) => {
                        println!("Adds url {} to block {}", url, block_name);
                        println!("NOTE: Please make sure the block name exists because ctk can't check if it exists in Cold Turkey. However, don't worry. There are no known errors when you give a block name that doesn't exist.");
                    },
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
                    },
                    Err(_) => {
                        println!("Cannot run the command: ctk toggle. Did you make sure Cold Turkey is installed and in the right folder? Try typing ctk");
                    }
                };
            }
            Command::Suggest => {
                suggest();
            }
        },
        None => {
            match cold_turkey.spawn() {
                Ok(_) => println!("Launches Cold Turkey!"),
                Err(_) => {
                    println!(r"Looks like you don't have Cold Turkey installed on C:\Program Files\Cold Turkey\Cold Turkey Blocker.exe");
                    println!("If you do have it installed, please put Cold Turkey Blocker.exe in the folder mentioned.");
                    println!("If not, you're welcome to download it at getcoldturkey.com.");
                }
            };
        }
    }
}
