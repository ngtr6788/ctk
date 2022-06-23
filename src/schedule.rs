use chrono::{NaiveTime, Timelike};
use clap::{Parser, Subcommand, ColorChoice};
use serde::Serialize;
use std::io;
use std::io::Write;

use crate::convert;

#[derive(Parser)]
#[clap(color = ColorChoice::Never)]
/// Scheduling for a Cold Turkey block
enum Schedule {
    /// Adds new scheduling blocks
    Add {
        /// Start of schedule block
        #[clap(parse(try_from_str = convert::str_to_time))]
        start_time: NaiveTime,
        /// End of schedule block
        #[clap(parse(try_from_str = convert::str_to_time))]
        end_time: NaiveTime,
        /// Applies on Sunday
        #[clap(long)]
        sun: bool,
        /// Applies on Monday
        #[clap(long)]
        mon: bool,
        /// Applies on Tuesday
        #[clap(long)]
        tue: bool,
        /// Applies on Wednesday
        #[clap(long)]
        wed: bool,
        /// Applies on Thursday
        #[clap(long)]
        thu: bool,
        /// Applies on Friday
        #[clap(long)]
        fri: bool,
        /// Applies on Saturday
        #[clap(long)]
        sat: bool,
        /// Applies on weekdays: same as --mon --tue --wed --thu --fri
        #[clap(long)]
        wkday: bool,
        /// Applies on weekends: same as --sat --sun
        #[clap(long)]
        wkend: bool,
        /// Applies on all days of the week
        #[clap(short, long)]
        all: bool,
        /// Decide if schedule block has no breaks, has allowance or has pomodoro
        #[clap(subcommand)]
        break_type: ScheduleBreak,
    },
    /// Edits one single schedule block
    Edit {
        /// Index / ID of the block
        #[clap(long)]
        id: usize,
        /// Edit day of the week: must be one of sun, mon, tue, wed, thu, fri, sat
        #[clap(parse(try_from_str = str_to_day))]
        day: Day,
        /// Edit start of schedule block
        #[clap(parse(try_from_str = convert::str_to_time))]
        start_time: NaiveTime,
        /// Edit end of schedule block
        #[clap(parse(try_from_str = convert::str_to_time))]
        end_time: NaiveTime,
        /// Decide if schedule block has no breaks, has allowance or has pomodoro
        #[clap(subcommand)]
        break_type: ScheduleBreak,
    },
    /// Remove schedule blocks
    Remove {
        /// A list of block IDs to delete
        ids: Vec<usize>,
        /// Delete all blocks
        #[clap(short, long)]
        all: bool,
    },
    /// Prints out all schedule blocks in JSON format
    Print,
    /// Saves schedule block and exits schedule
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
    Sat,
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
        _ => Err("Not a valid day of the week. Must be sun, mon, tue, wed, thu, fri, sat"),
    }
}

/// Sets if schedule block has no breaks, allowance or pomodoro
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

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ScheduleBlock {
    id: usize,
    start_time: String,
    end_time: String,
    #[serde(rename = "break")]
    break_type: String,
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

pub fn schedule(block_name: &str) -> Vec<ScheduleBlock> {
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
                let start_string_end = ",".to_owned()
                    + &start_time.hour().to_string()
                    + ","
                    + &start_time.minute().to_string();
                let end_string_end = ",".to_owned()
                    + &end_time.hour().to_string()
                    + ","
                    + &end_time.minute().to_string();

                let break_string = match break_type {
                    ScheduleBreak::Nobreak => "none".to_string(),
                    ScheduleBreak::Allowance { allowance_minutes } => allowance_minutes.to_string(),
                    ScheduleBreak::Pomodoro {
                        lock_minutes,
                        break_minutes,
                    } => lock_minutes.to_string() + "," + &break_minutes.to_string(),
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
            }
            Schedule::Edit {
                id,
                day,
                start_time,
                end_time,
                break_type,
            } => {
                if id >= final_schedule.len() {
                    println!("ID {} does not exist. Please choose an ID between 0 and {} inclusive", id, final_schedule.len() - 1);
                    continue;
                }
                let start_string_end = ",".to_owned()
                    + &start_time.hour().to_string()
                    + ","
                    + &start_time.minute().to_string();
                let end_string_end = ",".to_owned()
                    + &end_time.hour().to_string()
                    + ","
                    + &end_time.minute().to_string();

                let break_string = match break_type {
                    ScheduleBreak::Nobreak => "none".to_string(),
                    ScheduleBreak::Allowance { allowance_minutes } => allowance_minutes.to_string(),
                    ScheduleBreak::Pomodoro {
                        lock_minutes,
                        break_minutes,
                    } => lock_minutes.to_string() + "," + &break_minutes.to_string(),
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
            }
            Schedule::Print => {
                match serde_json::to_writer_pretty(io::stdout(), &final_schedule) {
                    Ok(_) => {}
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
