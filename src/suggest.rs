use chrono::{NaiveTime, Timelike};
use clap::{Parser, Subcommand, ColorChoice};
use rand::Rng;
use rpassword;
use serde::Serialize;
use serde_json;
use shlex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::{convert, schedule};

#[derive(Parser, Debug)]
#[clap(color = ColorChoice::Never)]
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
    Continuous { block_name: String },
    /// Schedules a block's blocking time over a week
    Schedule {
        /// Name of your wishlist Cold Turkey block (see suggest list)
        block_name: String,
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
pub enum LockMethod {
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
        #[clap(parse(try_from_str = convert::str_to_time))]
        /// Start of block range
        start_time: NaiveTime,
        #[clap(parse(try_from_str = convert::str_to_time))]
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
    schedule: Vec<schedule::ScheduleBlock>,
    custom_users: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
enum SchedType {
    Continuous,
    Scheduled,
}

#[derive(Debug, Serialize)]
struct Window {
    lock: bool,
    start_time: String,
    end_time: String,
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

pub fn suggest() {
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
                        LockMethod::Random => println!(
                            "Block {} locked by a string of random characters",
                            block_name
                        ),
                        LockMethod::Range => {
                            println!("Block {} locked within some time range", block_name)
                        }
                        LockMethod::Restart => {
                            println!("Block {} locked until restart", block_name)
                        }
                        LockMethod::Password => {
                            println!("Block {} locked with a password", block_name)
                        }
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
                            let window_string = if unlocked {
                                "unlock".to_owned()
                            } else {
                                "lock".to_owned()
                            };
                            let start_string = "@".to_owned()
                                + &start_time.hour().to_string()
                                + ","
                                + &start_time.minute().to_string();
                            let end_string = "@".to_owned()
                                + &end_time.hour().to_string()
                                + ","
                                + &end_time.minute().to_string();
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
                                println!(
                                    "Added {} to {} as a website exception",
                                    &path, block_name
                                );
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
                            println!(
                                "Added {} to {} as a Windows 10 application",
                                &path, block_name
                            );
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
                            println!(
                                "Windows 10 application {} removed from {}",
                                &path, &block_name
                            );
                        } else {
                            println!(
                                "Windows 10 application {} does not exist in {}",
                                &path, &block_name
                            );
                        }
                    }
                },
                None => println!("Block {} does not exist", &block_name),
            },
            Suggest::Continuous { block_name } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.sched_type = SchedType::Continuous;
                    println!(
                        "Made block {} to be blocked continously without schedule",
                        &block_name
                    );
                }
                None => println!("Block {} does not exist", block_name),
            },
            Suggest::Schedule { block_name } => match list_of_blocks.get_mut(&block_name) {
                Some(bs) => {
                    bs.sched_type = SchedType::Scheduled;
                    bs.schedule = schedule::schedule(&block_name);
                    println!("Added a schedule to block {}", &block_name);
                }
                None => println!("Block {} does not exist", block_name),
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
                    Ok(file) => match serde_json::to_writer_pretty(file, &list_of_blocks) {
                        Ok(_) => {
                            println!("Successfully saved to {} in current directory", display)
                        }
                        Err(why) => println!("Could not write to {}: {}", display, why),
                    },
                    Err(why) => {
                        println!("Could not create {}: {}", display, why);
                    }
                }
            }
            Suggest::Pwd => {
                if let Ok(current_dir) = env::current_dir() {
                    println!("{}", current_dir.display());
                }
            }
            Suggest::Quit => break,
        }
    }
}
