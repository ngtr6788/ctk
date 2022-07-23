# ctk
A command-line wrapper around [Cold Turkey Blocker](https://getcoldturkey.com)'s own command-line interface plus an interactive way to make Cold Turkey block JSON settings files called ctbbl files! Built in Rust and partly based on my [similar project built in Python](https://github.com/ngtr6788/Cold_PyTurkey).

## Commands
`ctk start <block_name>` - pretty self-explanatory: starts the given block if it is unlocked and disabled

`ctk start <block_name> for <minutes>` - starts the given block and locks it for a certain amount of minutes

`ctk start <block_name> until <time> [date]` - starts the block until the time (and optional date) given. 

Time can either be in 24-hour format (e.g. 6:30, 06:30, 18:30) or 12-hour format (e.g. 6:30pm, 6:30PM, 06:30am, 06:30AM). 

Date defaults to today if not given and can be in the following formats:
  - %d %B %Y (e.g. "07 Jun 1997", "09 July 2000", "18 September 2001")
  - %e %B %Y (e.g. "7 Jun 1997", "7 June 1997")
  - %B %d %Y (e.g. "Jun 07 1997", "June 07 1997")
  - %B %e %Y (e.g. "Jun 7 1997", "June 7 1997")
  - %F (e.g. 1997-07-07, not 1997-7-7)
  - %d/%m/%Y (e.g. 07/07/1997)
  
`ctk start <block_name> --password <password>` - starts the block and locks it with a password

`ctk stop <block_name>` - pretty self-explanatory: stops the block if it is unlocked

`ctk add <block_name> <url>` - adds a URL to the given block's 'blacklist', if you will

`ctk add --except <block_name> <url>` - adds a URL to the given block as an excpetion (or 'whitelist' if you will)

`ctk toggle <block_name>` - starts the block if it is not blocking, stops the block if it is unlocked and blocking

`ctk suggest` - opens a interactive interface to "suggest" and import new blocks to Cold Turkey Blocker on your computer

## Walkthrough on `ctk suggest` - WIP
[This is a work in progress. Things might be incomplete.]

When you type `ctk suggest`, this is what you are greeted with:

    Enter a new Cold Turkey block name:

Type in the name you wish to give it. You are then greeted with this:

    Choose a lock method:
    > No Lock
      Random Text
      Time Range
      Restart
      Password

You can use your arrow keys (up or down) or Vim-style navigation like J for down, K for up, to select and when you're done, press enter. You will then be prompted to configure the block settings custom to the block method.

You then get to add websites to the block, either in the "blacklist":

    Do you want to add websites to the blocklist? [y/n]
    Add a new website [press empty string to exit]: 

... or in the "whitelist" or exception list:
   
    Do you want to add websites to the exceptions list? [y/n]
    Add a new website [press empty string to exit]: 

This section is where the output is different for Windows and MacOS (at this point, the code is not configured for MacOS yet). For Windows, you can add executables (.exe files), folders containing .exe files, Windows 10 application and window titles. For MacOS, you can add applications, folders and binaries (though this is not configured for it yet)

### Windows 

Before adding executables or folders, you are greeted with this:

    Do you want to add executables or folders to the block? [y/n]

Then, you are greeted with something like this:

    [your current directory]
    >: 

You have the following commands:
- `cd [directory]` to change to the directory you're given
- `ls` to list all executables and folders in the current direction. This command also allows you to add the executables and folders in the current directory. 
- `search [keyword]` looks for all executables and folders in the current directory and all of its subfolders that approximately matches the keyword. WARNING: Can be slow
- `done`, `quit`, `q` when you're done

You now can add Windows 10 applications if you are on Windows, and you can select as many as you want

    Do you want to add Windows 10 applications or not? [y/n]
    Choose your Windows 10 apps:
    [ ] 3DViewer.exe
    [ ] AccountsControlHost.exe
    [ ] AddSuggestedFoldersToLibraryDialog.exe
    [ ] AppInstaller.exe
    [ ] ...
  
After Windows 10 apps, you can now add window titles.

    Do you want to block windows with certain titles? [y/n]
    Add a new window title [press empty string to exit]:

You can then schedule your blocks. This is what it looks like:

    Do you want to add a schedule to your blocks? [y/n]
    Do you want to add new schedule blocks? [y/n]
    Choose the times of the week applied:
    [ ] Sunday
    [ ] Monday
    [ ] ...
    Enter start time:
    Enter end time:

You are now done with one block! If you want to add additional blocks, you are welcome to do so. Otherwise, you can save them as a .ctbbl JSON file.

    Do you want to add new blocks? [y/n]
    Do you want to save these settings in a .ctbbl file? [y/n]
    Enter a new file name [empty string to create random name]: [type your file here]
    Successfully saved to [your file here].ctbbl in current directory

### MacOS

(the code is not configured for that yet. this is a work in progress)

## Wishlist
- [ ] Make this work on MacOS, since I don't have a MacOS device
- [ ] Extend this README.md by writing a walk-through for `ctk suggest`
- [ ] Improve on taking in datetime input 
- [ ] Work on handling Ctrl + C (Keyboard Interrupts)
- [ ] Improve on the interface for adding executables, applications and folders (user input and looks)
- [ ] Save `ctk suggest` progress if things go wrong so users can go back and continue where they left off
- [ ] Improve on `ctk start` password input
- [ ] Ask other people for any contributions, ideas, feedback, etc.
- [ ] Learn what a licence is and how to licence

## Licence
(no ideas what that is yet, but I don't have time to research it yet)