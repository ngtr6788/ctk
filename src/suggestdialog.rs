use crate::blocksettings::{AppString, Day, ScheduleBlock, ScheduleTime};
use crate::blocksettings::{BlockSettings, BreakMethod, LockMethod, RangeWindow, SchedType};
use crate::convert;
use crate::loop_dialoguer::LoopDialogue;
use chrono::NaiveTime;
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use shlex;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use sublime_fuzzy::{FuzzySearch, Match, Scoring};
use walkdir::WalkDir;

const WIN10_APPS: [&str; 99] = [
  "3DViewer.exe",
  "AccountsControlHost.exe",
  "AddSuggestedFoldersToLibraryDialog.exe",
  "AppInstaller.exe",
  "AppInstallerCLI.exe",
  "AppInstallerElevatedAppServiceClient.exe",
  "AppInstallerPythonRedirector.exe",
  "AppResolverUX.exe",
  "AssignedAccessLockApp.exe",
  "AuthenticationManager.exe",
  "BioEnrollmentHost.exe",
  "Calculator.exe",
  "CallingShellApp.exe",
  "CameraBarcodeScannerPreview.exe",
  "candycrushsaga.exe",
  "CapturePicker.exe",
  "CHXSmartScreen.exe",
  "Cortana.exe",
  "CredDialogHost.exe",
  "FileExplorer.exe",
  "FilePicker.exe",
  "GameBar.exe",
  "GameBar.exe",
  "GameBarElevatedFT.exe",
  "GameBarFTServer.exe",
  "GetHelp.exe",
  "HxAccounts.exe",
  "HxCalendarAppImm.exe",
  "HxOutlook.exe",
  "HxTsr.exe",
  "LocalBridge.exe",
  "LockApp.exe",
  "Maps.exe",
  "Microsoft.AAD.BrokerPlugin.exe",
  "Microsoft.AsyncTextService.exe",
  "Microsoft.ECApp.exe",
  "Microsoft.MicrosoftSolitaireCollection.exe",
  "Microsoft.Msn.News.exe",
  "Microsoft.Msn.Weather.exe",
  "Microsoft.Notes.exe",
  "Microsoft.Photos.exe",
  "Microsoft.Wallet.exe",
  "Microsoft.WebMediaExtensions.exe",
  "MixedRealityPortal.Brokered.exe",
  "MixedRealityPortal.exe",
  "Music.UI.exe",
  "myling.exe",
  "NarratorQuickStart.exe",
  "NcsiUwpApp.exe",
  "onenoteim.exe",
  "onenoteshare.exe",
  "OOBENetworkCaptivePortal.exe",
  "OOBENetworkConnectionFlow.exe",
  "PaintStudio.View.exe",
  "PeopleApp.exe",
  "PeopleExperienceHost.exe",
  "Photos.DLC.Main.exe",
  "Photos.DLC.MediaEngine.exe",
  "PilotshubApp.exe",
  "PinningConfirmationDialog.exe",
  "Print3D.exe",
  "ScreenClippingHost.exe",
  "ScreenSketch.exe",
  "SearchApp.exe",
  "SecHealthUI.exe",
  "SecureAssessmentBrowser.exe",
  "ShellExperienceHost.exe",
  "Skype.exe",
  "Solitaire.exe",
  "SoundRec.exe",
  "SpeechToTextOverlay64-Retail.exe",
  "Spotify.exe",
  "SpotifyMigrator.exe",
  "SpotifyStartupTask.exe",
  "StartMenuExperienceHost.exe",
  "StoreDesktopExtension.exe",
  "StoreExperienceHost.exe",
  "TCUI-App.exe",
  "TextInputHost.exe",
  "Time.exe",
  "UndockedDevKit.exe",
  "Video.UI.exe",
  "VideoProjectsLauncher.exe",
  "View3D.ResourceResolver.exe",
  "WebViewHost.exe",
  "WhatsNew.Store.exe",
  "Win32Bridge.Server.exe",
  "Win32WebViewHost.exe",
  "WindowsCamera.exe",
  "WindowsPackageManagerServer.exe",
  "WinStore.App.exe",
  "WpcUapApp.exe",
  "XBox.TCUI.exe",
  "XboxApp.exe",
  "XboxIdp.exe",
  "XGpuEjectDialog.exe",
  "YourPhone.exe",
  "YourPhoneAppProxy.exe",
  "YourPhoneServer.exe",
];

const TIMES_OF_WEEK: [&str; 7] = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
];

const LOCK_OPTIONS: [&str; 5] = [
  "No Lock",
  "Random Text",
  "Time Range",
  "Restart",
  "Password",
];

const ALLOWANCE_OPTIONS: [&str; 3] = ["No Breaks", "Allowance", "Pomodoro"];

fn best_match(query: &str, target: &str) -> Option<Match> {
  let scoring = Scoring::new(50, 0, 20, 0);
  return FuzzySearch::new(query, target)
    .score_with(&scoring)
    .case_insensitive()
    .best_match();
}

fn new_block_name() -> String {
  // Ask the user for the Cold Turkey block name
  Input::<String>::new()
    .with_prompt("Enter a new Cold Turkey block name")
    .loop_interact()
}

pub fn suggest() {
  let mut list_of_blocks: HashMap<String, BlockSettings> = HashMap::new();

  let mut continue_settings = true;

  while continue_settings {
    let block_name = new_block_name();
    if let Some(block_settings) = block_settings_from_stdin() {
      list_of_blocks.insert(block_name, block_settings);
    }

    continue_settings = Confirm::new()
      .with_prompt("Do you want to add new blocks?")
      .loop_interact();
  }

  let save_to_file = Confirm::new()
    .with_prompt("Do you want to save these settings in a .ctbbl file?")
    .loop_interact();

  if save_to_file {
    let file_name: String = Input::new()
      .with_prompt("Enter a new file name [empty string to create random name]")
      .allow_empty(true)
      .loop_interact();

    let final_file: String = if !file_name.is_empty() {
      format!("{}.ctbbl", file_name)
    } else {
      let num: u64 = rand::thread_rng().gen();
      format!("ctk_{}.ctbbl", num)
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
}

fn read_string_until_empty<S: Display>(prompt: S) -> Vec<String> {
  let mut string_vec = Vec::new();
  loop {
    let s: String = Input::new()
      .with_prompt(format!("{} [press empty string to exit]", prompt))
      .allow_empty(true)
      .loop_interact();

    if s.is_empty() {
      break;
    }

    string_vec.push(s);
  }

  string_vec
}

fn break_method_from_stdin() -> BreakMethod {
  // Ask the user if they want no breaks, allowance or pomodoro
  let allowance_method = Select::new()
    .with_prompt("Choose a break method")
    .items(&ALLOWANCE_OPTIONS)
    .loop_interact();

  match allowance_method {
    1 => {
      let allow_minutes: u32 = Input::new()
        .with_prompt("Enter allowance minutes")
        .loop_interact();

      BreakMethod::Allowance(allow_minutes)
    }
    2 => {
      let block_minutes: u32 = Input::new()
        .with_prompt("Enter block minutes")
        .loop_interact();

      let break_minutes: u32 = Input::new()
        .with_prompt("Enter break minutes")
        .loop_interact();

      BreakMethod::Pomodoro(block_minutes, break_minutes)
    }
    _ => BreakMethod::None,
  }
}

fn read_time_from_stdin<S: Into<String>>(prompt: S) -> NaiveTime {
  let mut input: Input<String> = Input::new();
  input.with_prompt(prompt);
  loop {
    let time_string: String = input.loop_interact();
    match convert::str_to_time(&time_string) {
      Ok(time) => break time,
      Err(_) => {
        eprintln!("Error in parsing the given time. Please try again.");
        continue;
      }
    }
  }
}

fn block_settings_from_stdin() -> Option<BlockSettings> {
  let mut block_settings = BlockSettings::new();

  // Ask the user to select a lock option
  let lock_method = Select::new()
    .with_prompt("Choose a lock method")
    .items(&LOCK_OPTIONS)
    .loop_interact();

  match lock_method {
    1 => {
      block_settings.lock = LockMethod::RandomText;

      let length: u32 = Input::new()
        .with_prompt("Enter a random string length")
        .loop_interact();

      block_settings.random_text_length = length;
    }
    2 => {
      block_settings.lock = LockMethod::Window;

      let start_time: NaiveTime = read_time_from_stdin("Enter start time");

      let end_time: NaiveTime = read_time_from_stdin("Enter end time");

      let lock_range: bool = Confirm::new()
        .with_prompt("Do you want to lock during that time range?")
        .loop_interact();

      block_settings.window = RangeWindow {
        lock_range,
        start_time,
        end_time,
      };
    }
    3 => {
      block_settings.lock = LockMethod::Restart;

      let restart_unblock: bool = Confirm::new()
        .with_prompt("Do you want the block to be unblocked after a restart?")
        .loop_interact();

      block_settings.restart_unblock = restart_unblock;
    }
    4 => {
      block_settings.lock = LockMethod::Password;

      let password = Password::new()
        .with_prompt("Enter a password")
        .loop_interact();

      block_settings.password = password;
    }
    _ => {
      block_settings.lock = LockMethod::None;
    }
  }

  block_settings.break_type = break_method_from_stdin();

  // Ask the user if they want add websites to the blocklist or not
  let website_block: bool = Confirm::new()
    .with_prompt("Do you want to add websites to the blocklist?")
    .loop_interact();

  if website_block {
    block_settings.web = read_string_until_empty("Add a new website");
  }

  // Ask the user if they want to add websites to the list of exceptions or not
  let website_exception: bool = Confirm::new()
    .with_prompt("Do you want to add websites to the exceptions list?")
    .loop_interact();

  // If so, continously add websites until user types empty string
  if website_exception {
    block_settings.exceptions = read_string_until_empty("Add a new website");
  }

  let app_block = Confirm::new()
    .with_prompt("Do you want to add executables or folders to the block?")
    .loop_interact();

  if app_block {
    let original_curdir = match env::current_dir() {
      Ok(dir) => match dir.to_str() {
        Some(dir_str) => dir_str.to_string(),
        None => {
          println!("Cannot print out the string of the current directory.");
          return None;
        }
      },
      Err(dir_err) => {
        println!("{}", dir_err);
        return None;
      }
    };
    loop {
      if let Ok(current_dir) = env::current_dir() {
        println!("{}", current_dir.display());

        let cmd_result: Result<String, std::io::Error> =
          Input::new().with_prompt(">").interact_text();

        if let Ok(cmd) = cmd_result {
          let shlex_parse: Vec<String> = match shlex::split(&cmd) {
            Some(parse) => parse,
            None => {
              println!("Cannot parse the command - please try again.");
              continue;
            }
          };

          if &shlex_parse[0] == "cd" {
            if shlex_parse.len() == 2 {
              let path = PathBuf::from(&shlex_parse[1]);
              env::set_current_dir(path);
            } else {
              env::set_current_dir(".");
            }
          } else if &shlex_parse[0] == "ls" {
            let searched_directory = match current_dir.read_dir() {
              Ok(dir) => dir,
              Err(err) => {
                println!("{}", err);
                continue;
              }
            };
            let apps_list: Vec<String> = searched_directory
              .filter_map(|e| e.ok())
              .filter(|e| e.path().extension().unwrap_or_default() == "exe" || e.path().is_dir())
              .filter(|e| e.path().to_str().is_some())
              .map(|e| e.path().to_str().unwrap().to_string())
              .collect();

            if !apps_list.is_empty() {
              let idxs = MultiSelect::new()
                .with_prompt(
                  "Which executable or folder would you like to add? [press space to select]",
                )
                .items(&apps_list)
                .loop_interact();

              idxs.into_iter().for_each(|i| {
                let s = apps_list[i].replace("\\", "/");
                let path = PathBuf::from(&s);
                if path.is_dir() {
                  block_settings.apps.push(AppString::Folder(s));
                } else if path.is_file() {
                  block_settings.apps.push(AppString::File(s));
                }
              });
            }
          } else if &shlex_parse[0] == "search" {
            if shlex_parse.len() == 2 {
              let keyword = &shlex_parse[1];
              let mut initial_count = 500;

              let find_progress_bar = ProgressBar::new(initial_count);

              find_progress_bar.set_style(
                ProgressStyle::default_bar()
                  .template("{wide_bar} Found {pos} executables and folders [ETA: {eta}]"),
              );
              find_progress_bar.println("Finding possible matches ...");

              let mut vec_exe = Vec::new();

              let mut exe_iterable = WalkDir::new(current_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().unwrap_or_default() == "exe" || e.path().is_dir())
                .filter(|e| e.path().to_str().is_some())
                .filter(|e| best_match(keyword, e.path().to_str().unwrap()).is_some())
                .map(|e| e.path().to_str().unwrap().to_string());

              loop {
                if let Some(exe) = exe_iterable.next() {
                  vec_exe.push(exe);
                  find_progress_bar.inc(1);
                } else {
                  break;
                }

                if vec_exe.len() as u64 > initial_count {
                  initial_count *= 2;
                  find_progress_bar.set_length(initial_count);
                }
              }

              find_progress_bar.finish_and_clear();

              if !vec_exe.is_empty() {
                let mut sort_initial_progress: u64 = 10000;
                let mut number_of_comparisons: u64 = 0;
                let sort_progress_bar = ProgressBar::new(sort_initial_progress);

                sort_progress_bar.println("Sorting matches by best match ...");
                sort_progress_bar.set_style(
                  ProgressStyle::default_bar().template("{pos} sorts done [ETA: {eta}]"),
                );

                vec_exe.sort_by(|a, b| {
                  sort_progress_bar.inc(1);
                  number_of_comparisons += 1;
                  if number_of_comparisons > sort_initial_progress {
                    sort_initial_progress *= 2;
                    sort_progress_bar.set_length(sort_initial_progress);
                  }
                  best_match(keyword, b)
                    .unwrap()
                    .cmp(&best_match(keyword, a).unwrap())
                });
                sort_progress_bar.finish_and_clear();

                let choose_exes = MultiSelect::new()
                  .with_prompt("Given the keyword, which executables do you want to block? [press space to select]")
                  .items(&vec_exe)
                  .loop_interact();

                choose_exes.into_iter().for_each(|i| {
                  let s = vec_exe[i].replace("\\", "/");
                  let path = PathBuf::from(&s);
                  if path.is_dir() {
                    block_settings.apps.push(AppString::Folder(s));
                  } else if path.is_file() {
                    block_settings.apps.push(AppString::File(s));
                  }
                });
              }
            }
          } else if &shlex_parse[0] == "done" || &shlex_parse[0] == "quit" || &shlex_parse[0] == "q"
          {
            break;
          }
        } else {
          eprintln!();
          continue;
        }
      }
    }

    env::set_current_dir(&original_curdir);
  }

  let win10_blocks = Confirm::new()
    .with_prompt("Do you want to add Windows 10 applications or not?")
    .loop_interact();

  if win10_blocks {
    let win10_choice = MultiSelect::new()
      .with_prompt("Choose your Windows 10 apps")
      .items(&WIN10_APPS)
      .loop_interact();

    win10_choice.into_iter().for_each(|i| {
      block_settings
        .apps
        .push(AppString::Win10(WIN10_APPS[i].to_string()));
    });
  }

  let allow_window_title = Confirm::new()
    .with_prompt("Do you want to block windows with certain titles?")
    .loop_interact();

  if allow_window_title {
    let mut window_titles: Vec<AppString> = read_string_until_empty("Add a new window title")
      .into_iter()
      .map(AppString::Title)
      .collect();
    block_settings.apps.append(&mut window_titles);
  }

  let schedule_block = Confirm::new()
    .with_prompt("Do you want to add a schedule to your blocks?")
    .loop_interact();

  if schedule_block {
    block_settings.sched_type = SchedType::Scheduled;
    loop {
      let add_sched = Confirm::new()
        .with_prompt("Do you want to add new schedule blocks?")
        .loop_interact();

      if !add_sched {
        break;
      }

      let time_of_week = MultiSelect::new()
        .with_prompt("Choose the times of the week applied")
        .items(&TIMES_OF_WEEK)
        .loop_interact();

      let start_time: NaiveTime = read_time_from_stdin("Enter start time");

      let end_time: NaiveTime = read_time_from_stdin("Enter end time");

      let break_type = break_method_from_stdin();

      time_of_week.into_iter().for_each(|i| {
        let day_of_week: Day = match i {
          0 => Day::Sun,
          1 => Day::Mon,
          2 => Day::Tue,
          3 => Day::Wed,
          4 => Day::Thu,
          5 => Day::Fri,
          6 => Day::Sat,
          _ => Day::Sun,
        };

        block_settings.schedule.push(ScheduleBlock {
          id: block_settings.schedule.len(),
          start_time: ScheduleTime {
            day_of_week,
            time: start_time,
          },
          end_time: ScheduleTime {
            day_of_week,
            time: end_time,
          },
          break_type: break_type.clone(),
        });
      });
    }
  } else {
    block_settings.sched_type = SchedType::Continuous;
  }

  Some(block_settings)
}
