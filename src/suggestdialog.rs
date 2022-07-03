#![allow(unused_variables)]

use dialoguer::{Input, Select, Confirm, MultiSelect, Password};
use walkdir::WalkDir;
use sublime_fuzzy::{Match, Scoring, FuzzySearch};
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::path::PathBuf;
use shlex;
use chrono::NaiveTime;

// NOTE TO SELF: I'm still merely prototyping here. That's why there are so many unwraps and allows stuff everywhere

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

const LOCK_OPTIONS: [&str; 5] = ["No Lock", "Random Text", "Time Range", "Restart", "Password"];

fn best_match(query: &str, target: &str) -> Option<Match> {
  let scoring = Scoring::new(50, 0, 20, 0);
  return FuzzySearch::new(query, target)
    .score_with(&scoring)
    .case_insensitive()
    .best_match();
}

pub fn suggest() {
  // Ask the user for the Cold Turkey block name
  let block_name: String = loop {
    match Input::<String>::new()
      .with_prompt("Enter a new Cold Turkey block name")
      .interact_text() {
        Ok(name) => break name,
        Err(_) => continue,
    }
  };

  // Ask the user to select a lock option  
  let lock_method = loop {
    match Select::new()
      .with_prompt("Choose a lock method")
      .items(&LOCK_OPTIONS)
      .interact() {
      Ok(method) => break method,
      Err(_) => continue,
    }
  };

  match lock_method {
    1 => {
      let length: i32 = loop {
        match Input::new()
          .with_prompt("Enter a random string length")
          .interact_text() {
            Ok(l) => break l,
            Err(_) => continue,
        }
      };
    } 
    2 => {
      let start_time: NaiveTime = loop {
        match Input::new()
        .with_prompt("Enter start time")
        .interact_text() {
          Ok(time) => break time,
          Err(_) => continue,
        }
      };

      let end_time: NaiveTime = loop {
        match Input::new()
        .with_prompt("Enter end time")
        .interact_text() {
          Ok(time) => break time,
          Err(_) => continue,
        }
      }; 
    }
    3 => {
      let restart_unblock: bool = loop {
        match Confirm::new()
        .with_prompt("Do you want the block to be unblocked after a restart?")
        .interact()
        {
          Ok(restart) => break restart,
          Err(_) => continue,
        }
      };
    }
    4 => {
      let password = loop { 
        match Password::new()
        .with_prompt("Enter a password")
        .interact() {
          Ok(pass) => break pass,
          Err(_) => continue,
        }
      };
    }
    _ => {}
  }

  // Ask the user if they want no breaks, allowance or pomodoro
  let allowance_options = ["No Breaks", "Allowance", "Pomodoro"];
  let allowance_method = loop {
    match Select::new()
    .with_prompt("Choose a break method")
    .items(&allowance_options)
    .interact() {
      Ok(opt) => break opt,
      Err(_) => continue,
    }
  };

  match allowance_method {
    1 => {
      let allow_minutes: i32 = loop {
        match Input::new()
        .with_prompt("Enter allowance minutes")
        .interact_text() {
          Ok(min) => break min,
          Err(_) => continue,
        }
      };
    }
    2 => {
      let block_minutes: i32 = loop {
        match Input::new()
        .with_prompt("Enter block minutes")
        .interact_text() {
          Ok(min) => break min,
          Err(_) => continue,
        }
      };
      let break_minutes: i32 = loop {
        match Input::new()
        .with_prompt("Enter break minutes")
        .interact_text() {
          Ok(min) => break min,
          Err(_) => continue,
        }
      };
    }
    _ => {}
  }

  // Ask the user if they want add websites to the blocklist or not
  let website_block: bool = loop {
    match Confirm::new()
    .with_prompt("Do you want to add websites to the blocklist?")
    .interact() {
      Ok(web_bloc) => break web_bloc,
      Err(_) => continue,
    }
  };
  
  if website_block {
    // Ask the user to add new websites
    loop {
      let website: String = loop {
        match Input::new()
        .with_prompt("Add a new website [press empty string to exit]")
        .allow_empty(true)
        .interact_text() {
          Ok(site) => break site,
          Err(_) => continue,
        }
      };
      if website.is_empty() {
        break;
      }
    }
  }

  // Ask the user if they want to add websites to the list of exceptions or not
  let website_exception: bool = loop {
    match Confirm::new()
    .with_prompt("Do you want to add websites to the exceptions list?")
    .interact() {
      Ok(web_bloc) => break web_bloc,
      Err(_) => continue,
    }
  };
  // If so, continously add websites until user types empty string
  if website_exception {
    loop {
      let website: String = loop {
        match Input::new()
        .with_prompt("Add a new website [press empty string to exit]")
        .allow_empty(true)
        .interact_text() {
          Ok(web) => break web,
          Err(_) => continue
        }
      };
      if website.is_empty() {
        break;
      }
    }
  }

  let app_block = loop {
    match Confirm::new()
    .with_prompt("Do you want to add executables or folders to the block?")
    .interact() {
      Ok(app_bloc) => break app_bloc,
      Err(_) => continue,
    }
  };

  if app_block {
    let mut apps_chosen: Vec<String> = Vec::new();

    let original_curdir = match env::current_dir() {
      Ok(dir) => {
        match dir.to_str() {
          Some(dir_str) => dir_str.to_string(),
          None => {
            println!("Cannot print out the string of the current directory.");
            return;
          }
        }
      }
      Err(dir_err) => {
        println!("{}", dir_err);
        return;
      }
    };
    loop {
      if let Ok(current_dir) = env::current_dir() {
        println!("{}", current_dir.display());

        let cmd_result: Result<String, std::io::Error> = Input::new()
          .with_prompt(">")
          .interact_text();

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
            let dir_to_look: PathBuf;
            if shlex_parse.len() == 2 {
              let path = PathBuf::from(&shlex_parse[1]);
              dir_to_look = path;
            } else {
              dir_to_look = env::current_dir().unwrap()
            }

            let searched_directory = match dir_to_look.read_dir() {
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
              
            if apps_list.len() != 0 {
              let idxs = match MultiSelect::new()
                .with_prompt("Which executable or folder would you like to add? [press space to select]")
                .items(&apps_list)
                .interact() {
                  Ok(indexes) => indexes,
                  Err(err) => {
                    println!("{}", err);
                    continue;
                }
              };

              for i in idxs {
                apps_chosen.push(apps_list[i].clone());
              }
            }
          } else if &shlex_parse[0] == "search" {          
            if shlex_parse.len() == 2 {
              let keyword = &shlex_parse[1];
              let mut initial_count = 500;
        
              let find_progress_bar = ProgressBar::new(initial_count);
              
              find_progress_bar.set_style(ProgressStyle::default_bar()
                .template("{wide_bar} Found {pos} executables and folders [ETA: {eta}]"));
              find_progress_bar.println("Finding possible matches ...");
        
              let mut vec_exe = Vec::new();
              
              let cur_dir = match env::current_dir() {
                Ok(dir) => dir,
                Err(err) => {
                  println!("{}", err);
                  continue;
                }
              };
              
              let mut exe_iterable = WalkDir::new(cur_dir).into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().unwrap_or_default() == "exe" || e.path().is_dir())
                .filter(|e| e.path().to_str().is_some())
                .filter(|e| best_match(&keyword, e.path().to_str().unwrap()).is_some())
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
              
              let mut sort_initial_progress: u64 = 10000;
              let mut number_of_comparisons: u64 = 0;
              let sort_progress_bar = ProgressBar::new(sort_initial_progress);

              sort_progress_bar.println("Sorting matches by best match ...");
              sort_progress_bar.set_style(ProgressStyle::default_bar()
                .template("{pos} sorts done [ETA: {eta}]"));
              
              vec_exe.sort_by(|a, b| {
                sort_progress_bar.inc(1);
                number_of_comparisons += 1;
                if number_of_comparisons > sort_initial_progress {
                  sort_initial_progress *= 2;
                  sort_progress_bar.set_length(sort_initial_progress);
                }
                return best_match(&keyword, b).unwrap().cmp(&best_match(&keyword, a).unwrap());
              });
              sort_progress_bar.finish_and_clear();
        
              let choose_exes = match MultiSelect::new()
                .with_prompt("Given the keyword, which executables do you want to block? [press space to select]")
                .items(&vec_exe)
                .interact() {
                  Ok(exes) => exes,
                  Err(err) => {
                    println!("{}", err);
                    continue;
                  }
              };
            }
          } else if &shlex_parse[0] == "done" || &shlex_parse[0] == "quit" || &shlex_parse[0] == "q" {
            break;
          }
        } else {
          break;
        }
      }
    }

    if let Ok(new_dir) = env::current_dir() {
      let new_current_dir = match new_dir.to_str() {
        Some(string) => string.to_string(),
        None => {
          println!("Cannot print out the string of the current directory.");
          return;
        }
      };

      let choices: [&String; 2] = [&new_current_dir, &original_curdir];

      let idx = loop {
        match Select::new()
        .with_prompt("Do you want to stay in this current directory or go back to the original directory you started?")
        .items(&choices)
        .interact() {
          Ok(i) => break i,
          Err(err) => {
            println!("{}", err);
            continue;
          }
        }
      };
      
      env::set_current_dir(&choices[idx]);
    }
  }

  let win10_blocks = loop {
    match Confirm::new()
    .with_prompt("Do you want to add Windows 10 applications or not?")
    .interact() {
      Ok(block) => break block,
      Err(err) => {
        println!("{}", err);
        continue;
      }
    }
  };

  if win10_blocks {
    let win10_choice = loop {
      match MultiSelect::new()
      .with_prompt("Choose your Windows 10 apps")
      .items(&WIN10_APPS)
      .interact() {
        Ok(choice) => break choice,
        Err(err) => {
          println!("{}", err);
          continue;
        }
      }
    };
  }

  let allow_window_title = loop {
    match Confirm::new()
    .with_prompt("Do you want to block windows with certain titles?")
    .interact() {
      Ok(allow) => break allow,
      Err(err) => {
        println!("{}", err);
        continue;
      }
    }
  };

  if allow_window_title {
    loop {
      let window: String = loop {
        match Input::new()
        .with_prompt("Add a new window title [press empty string to exit]")
        .allow_empty(true)
        .interact_text() {
          Ok(w) => break w,
          Err(err) => {
            println!("{}", err);
            continue;
          }
        }
      };
      if window.is_empty() {
        break;
      }
    }
  }

  let schedule_block = loop {
    match Confirm::new()
    .with_prompt("Do you want to add a schedule to your blocks?")
    .interact() {
      Ok(block) => break block,
      Err(err) => {
        println!("{}", err);
        continue;
      }
    }
  };

  if schedule_block {
    loop {
      let add_sched = loop {
        match Confirm::new()
        .with_prompt("Do you want to add new schedule blocks?")
        .interact() {
          Ok(add) => break add,
          Err(err) => {
            println!("{}", err);
            continue;
          }
        }
      };

      if !add_sched {
        break;
      }

      let time_of_week = match MultiSelect::new()
        .with_prompt("Choose the times of the week applied")
        .items(&TIMES_OF_WEEK)
        .interact() {
        Ok(time) => time,
        Err(err) => {
          println!("{}", err);
          continue;
        }
      };

      let start_time: NaiveTime = match Input::new()
        .with_prompt("Enter start time")
        .interact_text() {
        Ok(time) => time,
        Err(err) => {
          println!("{}", err);
          continue;
        }
      };

      let end_time: NaiveTime = match Input::new()
        .with_prompt("Enter end time")
        .interact_text() {
        Ok(time) => time,
        Err(err) => {
          println!("{}", err);
          continue;
        }
      };
    }
  }
}