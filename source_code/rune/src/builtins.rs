//! This file is part of RavnOS.
//!
//! RavnOS is free software:
//! you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation,
//! either version 3 of the License, or (at your option) any later version.
//!
//! RavnOS is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
//! without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//!
//!  You should have received a copy of the GNU General Public License along with RavnOS. If not, see <https://www.gnu.org/licenses/>

//!
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022-2023-2024

// Time lib
use core::time::Duration;

// Environment lib
use std::env;

// Path lib
use std::path::Path;
use std::path::PathBuf;

// Filesystem lib
use std::fs::{self, File};

// IO lib
use std::io::BufReader;
use std::io::{self, Read, Write};

// thread lib
use std::thread;

// Unix lib
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};

// Process lib
use std::process::Command;

// Time lib
use std::time::SystemTime;

// HashMap lib
use std::collections::HashMap;

// RavnOS libraries
extern crate libconfarg;
extern crate libfile;
extern crate libstream;

use libconfarg::RavnArguments;
use libfile::{decode_base64, which, RavnFile, RavnSizeFile};
use libstream::{file_filter, getprocs, search_replace_string, Epoch, Stream};

// Because this file is not a binary or lib, is just another module, to import
// under score another module we must use "crate"
use crate::io_mods::get_user_home;

// Here we use a const and not let because is a global variable
// As we know the size of each word we can use "&str" and then we specify the number
// of elements. This is because a const must have know size at compiling time.
const LBUILTINS: [&str; 39] = [
    "base64",
    "basename",
    "cd",
    "clear",
    "count",
    "cp",
    "date",
    "decodebase64",
    "disable_history",
    "du",
    "echoraw",
    "enable_history",
    "env",
    "exit",
    "expand",
    "false",
    "history",
    "head",
    "help",
    "home",
    "id",
    "join",
    "info",
    "mkdir",
    "mkfile",
    "move",
    "nl",
    "list",
    "ln",
    "ls",
    "proc",
    "pwd",
    "rm",
    "seq",
    "show",
    "sleep",
    "tail",
    "which",
    "$?",
];

const HBUILTINS: &str = "Help;
Remember respect the positions of each argument

_base64 [file] [file_n]: encode file/s into base64.
_basename: takes a path and prints the last filename.
_cd [PATH]: If path do not exist, goes to user home directory.
_clear: Clean the screen.
_count [file]: Show the file's number lines and words
_cp [source] [destination]: copy file or directory from [source] to [destination].
_date: display the current time and date in UTC-0 (which is the same that GTM-0).
_decodebase64 [input] [file]: decocde input from base64 to file.
_disable_history: disable save commands to history without truncate the file.
_du [path]: show disk usage ('du') in [path], read recusively.
_enable_history: enable save commands to history.
_echoraw: show string into stdout without interpreting special characters.
_env: show environment variables.
_exit: exit the shell properly.
_expand: convert tabs to spaces in file (with new file; [FILE]-edited), with '-t X' you can specify the spaces number, first the options (if exists) and then the file.
_false [option] : returns a false value, '-n' for rune native or '-u' for 1 (false in Unix and GNU).
_head -n [number] [file]: show [number] first lines for file.
_history: show the history commands with date and time.
_home: returns the current user's home directory.
_id [options]: show current user, '-n' for name and '-u' for UUID.
_info: show system's information.
_join [file_1] [file_n] [destination]: joins files into destionation file.
_mkdir [dest] : create directory if it has more subdirectories it will create them recursively.
_mkfile [file]: create empty file.
_move [source] [destination]: move files or directory to new location.
_nl [file]: prints each line with number.
_list: list builtins like this.
_ln [source] [dest]: creates a link [dest] to [source].
_ls [options] [path_1] [path_n]: lists files and directories in path.
_proc: show process using /proc directory
_pwd: print the current directory.
_rm [target]: delete the file or directory, if the directory have files inside must use '-r' argument to include them.
_seq [first]:[last]:[increment] : start a secuence from [first] to [last] using [increment] as increment.
_show [options] [file_1] [file_n]: show file's content, file's content in hexadecimal, system information or difference.
_sleep [seconds]:[nanoseconds] : waits X seconds with Y nanoseconds.
_tail [number] [file] : show the last [number] lines of [file].
_which [binary]: show where is located the binary based in PATH environment variable.
_$?: print the latest command exit return, not include builtins";

const RUNE_VERSION: &str = "v0.46.26";

// Builtins
// Are private for only be executed by rbuiltins

fn info() -> String {
    let os: &str = if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "freebsd") {
        "FreeBSD"
    } else if cfg!(target_os = "dragonfly") {
        "DragonFly BSD"
    } else if cfg!(target_os = "openbsd") {
        "OpenBSD"
    } else if cfg!(target_os = "netbsd") {
        "NetBSD"
    } else {
        "Unknown"
    };

    let user = {
        let mut varvalue: String = String::new();
        for (key, value) in std::env::vars() {
            if key == "USER" {
                varvalue = String::from(value);
                break;
            }
        }
        varvalue
    };

    let fileinfo = if Path::new("/etc/os-release").exists() {
        "/etc/os-release".to_string()
    } else {
        "/var/lib/os-release".to_string()
    };

    // Check if file exists
    let host_name = if Path::new("/etc/hostname").exists() {
        let mut file = match File::open("/etc/hostname") {
            Ok(d) => d,
            Err(e) => {
                println!("Error opening file; /etc/hostname");
                return e.to_string();
            }
        };
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_d) => {
                // When is moved to heap memory deletes the \n and \r
                buffer.pop();
                buffer
            }
            Err(_e) => {
                println!("Error reading /etc/hostname");
                format!("Error reading /etc/hostname")
            }
        }
    } else {
        "File /etc/hostname doesn't exists.".to_string()
    };

    // boot id
    let boot_id = if Path::new("/proc/sys/kernel/random/boot_id").exists() {
        let mut file = match File::open("/proc/sys/kernel/random/boot_id") {
            Ok(d) => d,
            Err(e) => {
                println!("Error opening file; /proc/sys/kernel/random/boot_id");
                return e.to_string();
            }
        };
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_d) => {
                // When is moved to heap memory deletes the \n and \r
                buffer.pop();
                buffer
            }
            Err(_e) => {
                println!("Error reading /proc/sys/kernel/random/boot_id");
                format!("Error reading /proc/sys/kernel/random/boot_id")
            }
        }
    } else {
        String::from("File /proc/sys/kernel/random/boot_id doesn't exist.")
    };

    let os_pretty = file_filter(&fileinfo, "PRETTY_NAME".to_string());
    let mut os_pretty = os_pretty[0].split('=');
    os_pretty.next();
    let os_pretty = match os_pretty.next() {
        Some(d) => d,
        None => return format!("Error getting pretty name"),
    };

    let os_url = file_filter(&fileinfo, "HOME_URL".to_string());
    let mut os_url = os_url[0].split('=');
    os_url.next();
    let os_url = match os_url.next() {
        Some(d) => d,
        None => return format!("Error getting HOME_URL name"),
    };

    let os_doc = file_filter(&fileinfo, "DOCUMENTATION_URL".to_string());
    let mut os_doc = os_doc[0].split('=');
    os_doc.next();
    let os_doc = match os_doc.next() {
        Some(d) => d,
        None => return format!("Error getting DOCUMENTATION_URL name"),
    };

    let os_legal = file_filter(&fileinfo, "PRIVACY_POLICY_URL".to_string());
    let mut os_legal = os_legal[0].split('=');
    os_legal.next();
    let os_legal = match os_legal.next() {
        Some(d) => d,
        None => return format!("Error getting PRIVACY_POLICY_URL name"),
    };

    let machine_id = if Path::new("/etc/machine-id").exists() {
        let mut file = match File::open("/etc/machine-id"){
            Ok(d) => d,
            Err(e) => {
                return format!("Error opening /etc/machine-id file; {e}")
            }
        };
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_d) => {
                // When is moved to heap memory deletes the \n and \r
                buffer.pop();
                buffer
            }
            Err(_e) => {
                println!("Error reading /proc/sys/kernel/random/boot_id");
                format!("Error reading /proc/sys/kernel/random/boot_id")
            }
        }
    } else {
        String::from("File /etc/machine-id doesn't exist.")
    };

    // Take CPU information from /proc/cpuinfo
    let cpuinfo = if Path::new("/proc/cpuinfo").exists() {
        file_filter(&"/proc/cpuinfo".to_string(), "model name".to_string())
    } else {
        "The /proc/cpuinfo file doesn't exists. CPU information unavailable."
            .to_string()
            .chars()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
    };
    let cpu = cpuinfo[0].clone();

    let cputhread = if Path::new("/proc/cpuinfo").exists() {
        file_filter(&"/proc/cpuinfo".to_string(), "processor".to_string()).len()
    } else {
        0
    };

    // As the information is stored in kB to transform into GB is needed
    // devide it.
    let memtotal = if Path::new("/proc/meminfo").exists() {
        let temp = file_filter(&"/proc/meminfo".to_string(), "MemTotal".to_string());
        let mut temp = temp[0].split("       ");
        temp.next();
        match temp.next() {
            Some(d) => d.to_string(),
            None => return format!("Error getting memory information"),
        }
    } else {
        "Memory information not available".to_string()
    };

    // Kernel CMD Line
    // Check if file exists
    let temp = match fs::read("/proc/cmdline") {
        Ok(d) => d,
        Err(_e) => return format!("Error reading /proc/cmdline"),
    };
    let kernelcmd = if Path::new("/proc/cmdline").exists() {
        match std::str::from_utf8(&temp) {
            Ok(d) => d.trim(),
            Err(e) => &format!("{e}"),
        }
    } else {
        "Kernel cmdline not available"
    };

    let kernel_version = if Path::new("/proc/version").exists() {
        let mut file = match File::open("/proc/version") {
            Ok(d) => d,
            Err(e) => {
                println!("Error opening file; /proc/version");
                return e.to_string();
            }
        };
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_d) => "",
            Err(_e) => "",
        };
        let mut buffer = buffer.split(" ");
        let kernel = match buffer.next() {
            Some(d) => d,
            None => &"No kernel detected",
        };
        buffer.next();
        format!("{}: {}",kernel, match buffer.next() {
                Some(d) => d,
                None => &"No version kernel detected",
            }
        )
    } else {
        String::from("Can not read kernel version.")
    };

    format!(" RavnOS's Shell\n Copyright 2023 Joaquin 'ShyanJMC' Crespo\n Rune Shell version; {RUNE_VERSION}\n OS: {os}\n OS Release: {os_pretty} \n OS url: {os_url} \n OS doc: {os_doc} \n OS legal: {os_legal} \n CPU: {cpu} \n CPU Thread: {cputhread} \n Memory: {memtotal} \n Machine ID: {machine_id} \n Hostname: {host_name} \n BOOT ID: {boot_id} \n BOOT/Kernel Command: {kernelcmd} \n Kernel version: {kernel_version} \n User: {user} \n")
}

fn base64(input: &String) -> Option<String> {
    let input: Vec<String> = input.split(' ').map(|e| e.to_string()).collect();
    let mut strreturn: String = String::new();
    for names in &input {
        let file = match fs::File::open(&names) {
            Ok(d) => d,
            Err(_e) => return None,
        };
        if input.len() > 1 {
            strreturn = strreturn + &format!("filename {names} base64 {{ {} }}\n\n", file.encode_base64());
        } else {
            strreturn = format!("base64 {{ {} }}\n", file.encode_base64());
        }
    }
    return Some(strreturn);
}

// Takes the path and returns only the file_name
fn basename(input: &String) -> Option<String> {
    // Path generation
    let _buff = Path::new(&input);
    // If "file_name" type returns is equal to Some(X) do that
    if let Some(filename) = _buff.file_name() {
        // To avoid use "unexpect" or "unwrap"
        return Some(
            match filename.to_str() {
                Some(d) => d,
                None => "",
            }
            .to_string(),
        );
    }
    None
}

// As this function do not return Err(e) does not matter if (e) is static str becuase
// never will exist
fn date() -> Result<String, &'static str> {
    let convertion = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH){
        Ok(d) => (d.as_secs() as i64).epoch_to_human(),
        Err(_e) => {
            return Err(&"Error getting SystemTime since UNIX_EPOCH");
        },
    };
    Ok(format!( "{convertion}"))
}

fn decodebase64(input: &String) -> Option<String> {
    let input: Vec<String> = input.split(' ').map(|e| e.to_string()).collect();
    if input.len() < 2 {
        eprintln!("Few arguments; [base64] [file_target]");
        return None;
    }
    let buffer = match input.get(0) {
        Some(d) => d,
        None => return None,
    };
    let mut file = match File::create(match input.get(1) {
        Some(d) => d,
        None => return None,
    }) {
        Ok(d) => d,
        Err(_e) => return None,
    };

    let output = match decode_base64(&buffer){
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error decoding base64 flow.\n\t{e}");
            return None
        }
    };

    match file.write_all(&output) {
        Ok(_d) => return Some(format!("{:?}: Saved correctly", input.get(1))),
        Err(_e) => return None,
    }
}

fn disk_usage(input: &String) -> Option<String> {
    let input = input.clone();

    if input.split(' ').map(|e| e.to_string()).collect::<Vec<String>>().len() > 1 {
        return Some("Too many arguments. Pass just one path".to_string());
    } else if input.len() == 0 {
        return Some("Too few arguments. Pass just one path".to_string());
    }

    let fs_struct = input.readdir_recursive();
    let mut temp_buff = HashMap::new();
    let mut result = String::new();

    for dir in fs_struct.dbuff {
        let size = match fs::metadata(dir.clone()){
            Ok(d) => d.size().size_to_human(),
            Err(_e) => format!("Error getting directory's metadata"),
        };
        temp_buff.insert(size, dir);
    }

    for file in fs_struct.fbuff {
        let size = match fs::metadata(file.clone()){
            Ok(d) => d.size().size_to_human(),
            Err(_e) => format!("Error getting file's metadata"),
        };
        temp_buff.insert(size, file);
    }

    for (k, i) in temp_buff {
        result = result + &format!("{k} {i}\n");
    }

    if result.is_empty() {
        return None;
    } else {
        return Some(result);
    }
}

fn environmentvar() -> String {
    let mut buffer: Vec<String> = Vec::new();
    let mut buffer2: String = String::new();
    for (key, value) in std::env::vars() {
        let envv = format!("{}; {}\n", key, value);
        buffer.push(envv);
    }
    for data in buffer {
        buffer2 = buffer2 + &data.to_string();
    }
    buffer2
}

fn expand(input: String) -> String {
    if input.contains("-t") {
        let s_number: usize;
        let args = input.split(' ').collect::<Vec<&str>>();
        if args[0] == "-t" {
            s_number = match args[1].trim().parse(){
                Ok(d) => d,
                Err(e) => return format!("Error {e}")
            };
            let mut file = match File::open(args[2]) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error opening file; {e}");
                    return String::from("Error opening file.");
                }
            };
            let mut string = String::new();
            let _ = file.read_to_string(&mut string);
            let ninput: String =
                match search_replace_string(&string, &'\t'.to_string(), &" ".repeat(s_number)) {
                    Ok(d) => d,
                    Err(_e) => String::from("Matching not found"),
                };

            let nfile = args[2].to_string() + "-edited";
            match mkfile(Path::new(&nfile)) {
                Ok(_d) => (),
                Err(e) => {
                    eprintln!("Error creating file; {e}");
                    return String::from("Error creating file.");
                }
            };
            match fs::write(nfile.clone(), ninput) {
                Ok(_d) => println!("Writted new string into new file; {nfile}"),
                Err(e) => {
                    eprintln!("Error writting new file; {e}");
                    return String::from("Error writting file.");
                }
            };
        }
        "".to_string()
    } else {
        let mut file = match File::open(input.clone()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error opening file; {e}");
                return String::from("Error opening file.");
            }
        };
        let mut string = String::new();
        let _ = file.read_to_string(&mut string);
        let ninput: String =
            match search_replace_string(&string, &'\t'.to_string(), &"        ".to_string()) {
                Ok(d) => d,
                Err(_e) => String::from("Matching not found"),
            };

        let nfile = input + "-edited";
        match mkfile(Path::new(&nfile)) {
            Ok(_d) => (),
            Err(e) => {
                eprintln!("Error creating file; {e}");
                return String::from("Error creating file.");
            }
        };
        match fs::write(nfile.clone(), ninput) {
            Ok(_d) => println!("Writted new string into new file; {nfile}"),
            Err(e) => {
                eprintln!("Error writting new file; {e}");
                return String::from("Error writting file.");
            }
        };
        "".to_string()
    }
}

fn count(input: &String) -> String {
    let mut output;
    output = format!("Lines {{ {} }} \n", match fs::read_to_string(input){
        Ok(d) => d.lines().count(),
        Err(e) => return format!("Error reading file; {e}"),
    });
    output = output + &format!("Words - Letters {{ {:?} }} \n", match fs::read_to_string(input){
        Ok(d) => d.word_count(),
        Err(e) => return format!("Error reading file; {e}"),
    });
    output
}

fn cd(path: String) -> () {
    if path.is_empty() {
        // Goes to home user dir
        let binding = get_user_home();
        let home: &str = binding.as_str();
        match env::set_current_dir(&home) {
            Ok(d) => d,
            Err(_e) => {
                eprintln!("Fail changing to current home directory");
            }
        }
    } else if path != ".." {
        let buff = path.trim();
        let npath = Path::new(&buff);
        match env::set_current_dir(&npath) {
            Ok(d) => d,
            Err(_e) => eprintln!("Failing setting the new working path"),
        }
    } else {
        // I know, I know, I also do not like to many methods but
        // PathBuf type is....complicated
        let mut buff: Vec<String> = match env::current_dir() {
            Ok(d) => match d.into_os_string().into_string(){
                Ok(d) => d.split("/").map(|e| e.to_string()).collect(),
                Err(_e) => {
                    eprintln!("Failing getting current dir");
                    return ();
                }
            },
            Err(_e) => {
                eprintln!("Failing getting current dir");
                return ();
            },
        };

        buff.remove(buff.len() - 1);
        let npath = {
            let mut vtemp = String::new();
            for i in buff {
                if vtemp.is_empty() {
                    vtemp = "/".to_owned() + &i;
                } else {
                    vtemp = vtemp + &"/" + &i;
                }
            }
            vtemp
        };
        match env::set_current_dir(npath) {
            Ok(d) => d,
            Err(_e) => eprintln!("Failing setting the new working path"),
        }
    }
}

fn clear() {
    // \x1B[ : ASCII scape character and start control secuence
    // \x1B[2J: Clears the entire screen
    // \x1B[<n>;<m>H: Moves the cursor to row <n> and column <m>
    // \x1B[<n>A: Moves the cursor <n> lines up
    // \x1B[<n>B: Moves the cursor <n> lines down
    // \x1B[<n>C: Moves the cursor <n> columns right
    // \x1B[<n>D: Moves the cursor <n> columns left
    // \x1B[K: Clears from the cursor position to the end of the line
    // \x1B[<n>J: Clears the screen from the cursor position to the end of the screen if <n> is 0, from the beginning of the screen to the cursor position if <n> is 1, and clears the entire screen if <n> is 2
    // \x1B[<n>m: Sets the text style. <n> can be 0 (sets style to normal), 1 (sets style to bold), 2 (sets style to dim), 3 (sets style to italic), 4 (sets style to underline), 5 (sets style to blinking), 7 (inverts the foreground and background colors), 8 (hides the text), 22 (disables bold or dim style), 23 (disables italic style), 24 (disables underline style), 25 (disables blinking style), 27 (disables color inversion) and 28 (shows hidden text).
    // \x07: Emits a beep or alert sound
    print!("\x1B[1;1H\x1B[2J");
}

fn echoraw(input: &String) -> String {
    input.clone()
}
// This function not copy directly using the kernel's filesystem
// to avoid any possible issue we copy bit a bit directly.
fn copy<'a>(source: &Path, dest: &Path) -> Result<(), &'a str> {
    if dest.exists() {
        let str: &str = "Destination already exists";
        Err(str)
    } else {
        // With the variables "sd" and "dd" we take the complete path,
        // we split them and then we take the last directory

        // Source directory
        let sd: String = {
            // For relative path
            if source.display().to_string().len() != 1 {
                //let temp = source.display().to_string();
                //let buffer: Vec<&str> = temp.split('/').collect();

                //let mut temp = buffer.clone();
                //temp.remove( (temp.len()-1) );
                //drop(buffer);

                // We just not take the last
                //temp.iter().map(|e| e.to_string() + &"/".to_string() ).collect()
                source.display().to_string().trim().to_string()
            } else {
                match env::current_dir() {
                    Ok(d) => format!(
                        "{}/{}",
                        d.display().to_string(),
                        source.display().to_string()
                    ),
                    Err(e) => {
                        eprintln!("{e}");
                        return Err("Error getting current dir for source");
                    }
                }
            }
        };

        // Destination directory
        let dd: String = {
            let temp = dest.display().to_string();
            let buffer: Vec<&str> = temp.split('/').collect();
            // For absolute path
            if buffer.len() > 1 {
                format!("{}", dest.display().to_string())
            }
            // For relative path
            else {
                match env::current_dir() {
                    Ok(d) => format!("{}/{}", d.display().to_string(), buffer[0]),
                    Err(e) => {
                        eprintln!("{e}");
                        return Err("Error getting current dir for destionation");
                    }
                }
            }
        };

        // To detect if "source" is a file
        if source.is_file() {
            // Open file
            let mut b_file = match File::open(source.display().to_string()) {
                Ok(d) => d,
                Err(_e) => {
                    let str: &str = "Fail opening source file/dir";
                    return Err(str);
                }
            };

            // Read it saving to b_reader
            let mut b_reader = Vec::new();
            match b_file.read_to_end(&mut b_reader) {
                Ok(_d) => {}
                Err(_e) => {
                    let str: &str = "Error reading source file, check permissions.";
                    return Err(str);
                }
            };

            // Create the file
            match File::create(dest.display().to_string()) {
                Ok(_d) => {}
                Err(_e) => {
                    print!("{_e}; {}", dest.display());
                    let str: &str = "Error creating destination";
                    return Err(str);
                }
            }

            // Write, as b_reader now has bytes, will save it in the same way avoiding possible issues
            match fs::write(dest, b_reader) {
                Ok(d) => d,
                Err(_e) => {
                    let str: &str = "Error writting buffer to destination file, check permissions.";
                    return Err(str);
                }
            }

            // If is not, is a directory
        } else if source.is_dir() {
            let entries = source.display().to_string().readdir_recursive();

            for d in &entries.dbuff {
                // Takes the absolute path of directory in variable "d", search the directories
                // of old path unless that last (which is the directory to copy) and replace it with
                // the absolute path of new path with variable "dd"
                let ddir = match search_replace_string(&d, &sd, &dd) {
                    Ok(d) => d,
                    Err(_e) => {
                        eprintln!("{_e}");
                        continue;
                    }
                };

                match mkdir_r(Path::new(&ddir)) {
                    Ok(_d) => {},
                    Err(e) => eprintln!("{e}"),
                }
            }

            for d in &entries.fbuff {
                let npath: String = match search_replace_string(&d, &sd, &dd) {
                    Ok(d) => d,
                    Err(_e) => {
                        eprintln!("{_e}");
                        continue;
                    }
                };

                // Open file
                let mut b_file = match File::open(d) {
                    Ok(d) => d,
                    Err(_e) => {
                        let str: &str = "Fail opening source file/dir";
                        return Err(str);
                    }
                };

                // Read it saving to b_reader
                let mut b_reader = Vec::new();
                match b_file.read_to_end(&mut b_reader) {
                    Ok(_d) => {},
                    Err(_e) => {
                        let str: &str = "Error reading source file, check permissions.";
                        return Err(str);
                    }
                };

                // Create the file
                match File::create(npath.clone()) {
                    Ok(_d) => {},
                    Err(_e) => {
                        println!("{npath} {{ {_e} }}");
                        let str: &str = "Error creating destination";
                        return Err(str);
                    }
                }

                // Write, as b_reader now has bytes, will save it in the same way avoiding possible issues
                match fs::write(npath.clone(), b_reader) {
                    Ok(d) => d,
                    Err(_e) => {
                        let str: &str =
                            "Error writting buffer to destination file, check permissions.";
                        return Err(str);
                    }
                }
            }
        }

        Ok(())
    }
}

fn ffalse(input: &String) -> Result<(), bool> {
    if !input.contains("-n") && !input.contains("-u") {
        eprintln!("Bad arguments; -n or -u");
        return Err(false);
    } else if input.contains("-n") {
        println!("false");
        return Ok(());
    } else if input.contains("-u") {
        println!("1");
        return Ok(());
    }
    return Err(false);
}

fn head(input: &String) {
    let file;
    let mut fdata = Default::default();
    let mut lnumber = 0;
    let mut cnumber = 1;
    let s_lnumber: Vec<&str> = input.split(' ').collect();
    if input.contains("-n") && s_lnumber.len() <= 2 {
        eprintln!("Not enough arguments. Remember; head -n [number] [file]");
        return;
    }
    if input.contains("-n") {
        lnumber = match s_lnumber[1].trim().parse() {
            Ok(d) => d,
            Err(e) => {
                eprintln!(
                    "Error parsing str to int; {e}\nVerify kernel compatiblity with Rust std"
                );
                return;
            }
        };
    }
    file = match File::open(s_lnumber[2].trim()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error opening file; {e}");
            return;
        }
    };

    drop(s_lnumber);

    // Is more eficient that a raw read
    let mut buff = BufReader::new(&file);
    let _ = buff.read_to_string(&mut fdata);
    drop(file);
    for i in fdata.lines() {
        if cnumber <= lnumber {
            println!("{i}");
            cnumber += 1;
        } else {
            return;
        }
    }
}

fn envhome<'a>() -> Result<String, &'a str> {
    Ok(get_user_home())
}

fn id(input: &String) -> Result<String, String> {
    let mut buff = String::new();
    let username = match env::var("USERNAME") {
        Ok(d) => d,
        Err(e) => e.to_string(),
    };
    if input.contains("-n") {
        buff = username.to_string();
    }
    if input.contains("-u") {
        let uuid_line = file_filter(&"/etc/passwd".to_string(), username.clone());
        // The return is a Vec<&str> we take the [1] position
        let uuid = (uuid_line[0].split(':').collect::<Vec<&str>>())[2];
        let guid = (uuid_line[0].split(':').collect::<Vec<&str>>())[3];
        println!("userid {{ {uuid} }} ");
        println!("groupid {{ {guid} }} ");
    }

    if input.contains("-g") {
        let guid_line = file_filter(&"/etc/group".to_string(), username);
        let mut groups = String::new();
        for i in guid_line {
            let name = (i.split(':').collect::<Vec<&str>>())[0];
            let id = (i.split(':').collect::<Vec<&str>>())[2];
            groups = groups + &name + ":" + &id + "\n";
        }

        println!("groups; {{ {groups} }}");
    }

    if !input.contains("-u") && !input.contains("-n") && !input.contains("-g") {
        println!("Usage; '-n' for name, '-g' for user groups and '-u' for UUIDs");
    }
    Ok(buff)
}

fn join(input: &String) -> Result<(), &str> {
    if input.len() <= 2 {
        return Err("Not enough arguments");
    }
    let mut files: Vec<&str> = input.split_whitespace().collect();
    let lenght = files.len();
    let mut destination = match File::create(files[lenght - 1]) {
        Ok(d) => d,
        Err(_e) => {
            return Err("Error creating destination file");
        }
    };
    let mut fdata = String::new();
    files.remove(lenght - 1);

    for i in files {
        let file = match File::open(i.trim()) {
            Ok(d) => d,
            Err(_e) => {
                return Err("Error opening file");
            }
        };
        let mut buff = BufReader::new(&file);
        let _ = buff.read_to_string(&mut fdata);
        drop(file);
    }
    match destination.write_all(fdata.as_bytes()) {
        Ok(_d) => return Ok(()),
        Err(_e) => {
            return Err("Error writting destination file from buffer");
        }
    };
}

fn ln(source: &Path, dest: &Path) -> Result<String, ()> {
    match symlink(source, dest) {
        Ok(_d) => Ok(format!(
            "Symlink created for {} pointing to {}",
            dest.display(),
            source.display()
        )),
        Err(_e) => Err(()),
    }
}

fn ls(input: &String) -> String {
    // env::args() takes program's arguments
    // collect() takes arguments and returns in tuple
    let arguments: Vec<String> = input.split(' ').map(|e| e.to_string()).collect();

    // Result buffer
    let mut buffer: Vec<String> = Vec::new();

    // Return buffer
    let mut returnbuff = String::new();

    // Init the configuration as clean
    let mut config = libconfarg::LsConfiguration {
        verbose: false,
        proc: false,
        lines: false,
        clean: false,
    };

    if arguments.checkarguments_help("ls") {
        //returnbuff = format!(" ");
        return " ".to_string();
    }

    // The vec<String> return with files index is stored in "lists" variable.
    // The method is from RavnArguments trait.
    let mut options: Vec<&str> = Vec::new();
    let mut lists: Vec<String> = arguments.check_arguments("ls", &mut options);

    if lists.is_empty() || lists.contains(&"".to_string()) || lists.contains(&".".to_string()) {
         lists.push(match env::current_dir(){
             Ok(d) => d.display().to_string(),
             Err(_e) => format!(" "),
         });
    }

    if lists.contains(&"".to_string()) {
        let index = match lists.iter().position(|e| *e == ""){
            Some(d) => d,
            None => {
                return "Error getting position in ls".to_string();
            }
        };
        lists.remove(index);
    } else if lists.contains(&".".to_string()) {
        let index = match lists.iter().position(|e| *e == "."){
            Some(d) => d,
            None => {
                return "Error getting position in ls".to_string();
            }
        };
        lists.remove(index);
    }

    for confs in options {
        if confs == "verbose" {
            config.verbose = true;
        } else if confs == "proc" {
            config.proc = true;
        } else if confs == "lines" {
            config.lines = true;
        } else if confs == "clean" {
            config.clean = true;
        }
    }

    if config.proc {
        let procs: Vec<String> = getprocs();
        for strings in procs {
            returnbuff = returnbuff + &format!("\n{strings}");
        }
        return returnbuff;
    }

    for names in &lists {
        // Entries store the files and directories inside path.
        let mut entries = Vec::new();
        // File buffer is used to store the file name if argument is not a dir.
        // Check if arguments is directory.

        match Path::new(names).is_dir() {
            true => {
                entries = names.readdir();
                // I must use here and not as "readdir" method because if not the type will be forced to
                // "()" (which is more like a unit and at the same time is a type also).
                entries.sort();
            }
            false => {
                // Check if arguments is a file.
                if Path::new(names).is_file() {
                    // If is a file, store it in variables.
                    entries.push(PathBuf::from(names));
                }
            }
        }

        if config.lines && !config.clean {
            if !config.verbose {
                returnbuff =
                    returnbuff + &format!("\nList of elements in {}; {}", names, &entries.len());
                return returnbuff;
            } else {
                returnbuff =
                    returnbuff + &format!("\nList of elements in {}; {}\n", names, &entries.len());
            }
        }

        if config.verbose && !config.clean {
            // Here I found an issue; as "readdir" returns Vec<PathBuf>, "metadata" will have
            // issues in some paths like; ~/ . Because of that, we must convert it to String.
            for h in &entries {
                // 1; the name of file/directory
                // 2; the time modified
                // 3; the permissions (in octal)
                // 4: the owner
                // 5: the size

                // Here "unwrap()" is not a good option because can fail, for example; if some symlink not points properly
                // because of it I use a check.

                if !h.exists() {
                    // eprintln! shows string in stderr
                    eprintln!(
                        "{}: File/dir/symlink do not exist, is invalid or is broken.",
                        h.display()
                    );
                    // break the loop for current stage
                    continue;
                }

                let fmetadata = match fs::metadata(h.display().to_string()){
                    Ok(d) => d,
                    Err(e) => return format!("Error getting metadata; {e}"),
                };

                // ID numeric to user
                let ownerout = match Command::new("/usr/bin/id").arg(fmetadata.uid().to_string()).output(){
                    Ok(d) => d,
                    Err(e) => return format!("Error getting uid; {e}"),
                };
                // When you use "output" method, the stdout of command will be stored in
                // "stdout" field. But, is stored as u8, and needs to be processed as utf8.

                let owner = match std::str::from_utf8(&ownerout.stdout) {
                    Err(_e) => format!("Error reading owner, check file/dir permissions."),
                    Ok(d) => match d.strip_suffix('\n') {
                        Some(d) => {
                            let buffer = d;
                            let buffer2 = buffer
                                .split(' ')
                                .map(|e| e.to_string())
                                .collect::<Vec<String>>();
                            format!("{} {}", buffer2[0], buffer2[1])
                        }
                        None => format!("Error reading owner, check file/dir permissions."),
                    },
                };

                // We convert "h" to Path type, we get the last file/dir name and we convert it to static str.
                let df_name = match Path::new(h).file_name(){
                    Some(d) => match d.to_str() {
                        Some(d) => d,
                        None => &"Error getting path's filename",
                    }
                    None => &"Error getting path's filename"
                };

                let varpermissions = {
                    let mut temp = String::new();
                    for i in format!("{:o}", fmetadata.permissions().mode()).permission_to_human() {
                        temp = temp + &i;
                    }
                    temp
                };

                returnbuff = returnbuff
                    + &format!(
                        "{} \t[{}]\t[{}]\t[{}] {}\n",
                        if h.is_symlink() {
                            format!("s: {df_name} -> {}", match std::fs::read_link(h.clone()){
                                    Ok(d) => d.display().to_string(),
                                    Err(_e) => format!("Error getting link information"),
                                }
                            )
                        } else if h.is_file() {
                            format!("f: {df_name}")
                        // Why not use "else" directly?
                        // because maybe there are one error
                        // with the inode and is not correctly identified
                        } else if h.is_dir() {
                            format!("d: {df_name}/")
                        } else {
                            format!("?: {df_name}")
                        },
                        fmetadata.mtime().epoch_to_human(),
                        // Permissions
                        // Permissions method by default will return in bits, if you want the octal chmod
                        // syntax need to use ".mode()".
                        // As Octal is not a type by it self, we need use "format!" macro to convert it in
                        // octal mode, the return is a String.
                        varpermissions,
                        owner,
                        fmetadata.size().size_to_human()
                    );
            }
            // Show filename and size.
            for ee in &buffer {
                returnbuff = returnbuff + &format!("{}\n", ee);
            }
        } else {
            if !config.clean && lists.len() > 1 {
                returnbuff = returnbuff + &format!("{names} {{ \n{}\n }}\n", &entries.iter().map(|e| e.display().to_string() + "\n").collect::<String>().trim() );
            } else {
                returnbuff = returnbuff + &format!( "{names} {{ \n{}\n }}",&entries.iter().map(|e| e.display().to_string() + "\n").collect::<String>().trim() );
            }
        }

        // I must do this because if I use "clean()" will be dropped from memory.
        buffer = Vec::new();
    }
    returnbuff
}

// Create a directory recusively
// We use "Path" type because it returns absolute path
fn mkdir_r(path: &Path) -> Result<u64, String> {
    if path.display().to_string().is_empty() {
        Err("Help;\n mkdir [directory]".to_string())
    } else {
        match std::fs::create_dir_all(path.display().to_string()) {
            Ok(_d) => Ok(0),
            Err(e) => return Err(e.to_string()),
        }
    }
}

fn mkfile(path: &Path) -> Result<(), &str> {
    if !path.exists() {
        match std::fs::File::create(path.display().to_string()) {
            Ok(_d) => return Ok(()),
            Err(_e) => return Err(
                "Fail to create file, verify if path exists and if you have the right permissions.",
            ),
        }
    } else {
        Err("File already exists.")
    }
}

fn fdmove(input: String) {
    let arguments: Vec<_> = input.split(' ').collect();
    let n_arguments = arguments.len();
    let destination = arguments[n_arguments - 1];
    let mut source: Vec<String> = Vec::new();

    for i in &arguments[0..(n_arguments - 1)] {
        source.push(i.to_string());
    }

    if Path::new(destination).is_dir() {
        let mut ndestination: Vec<String> = Vec::new();
        for i in &source {
            let snumber: Vec<_> = i.split('/').collect();
            let sbuffer = snumber.len();
            ndestination.push(destination.to_string() + match snumber.iter().nth(sbuffer - 1){
                Some(d) => d,
                None => " ",
            });
        }
        for i in &source {
            for j in &ndestination {
                match copy(Path::new(&i), Path::new(&j)) {
                    Ok(_d) => {
                        let _ = remove_f_d(i.clone());
                        ()
                    }
                    Err(e) => eprintln!("{e}"),
                };
            }
        }
    } else {
        for i in &source {
            match copy(Path::new(&i), Path::new(&destination)) {
                Ok(_d) => {
                    let _ = remove_f_d(i.to_string());
                    ()
                }
                Err(e) => eprintln!("{e}"),
            };
        }
    }
}

fn number_line(input: &String) -> Result<(), &str> {
    let mut lnumber = 0;
    let mut fdata = String::new();
    let file = match File::open(Path::new(input.trim())) {
        Ok(d) => d,
        Err(_e) => return Err("Error opening file, check permissions and file system"),
    };
    let mut buff = BufReader::new(&file);
    let _ = buff.read_to_string(&mut fdata);
    drop(file);
    for i in fdata.lines() {
        lnumber += 1;
        println!("{lnumber}   {i}");
    }
    Ok(())
}

fn proc() -> Result<String, String> {
    let procs: Vec<String> = getprocs();
    let mut strreturn = String::new();
    for strings in procs {
        strreturn = strreturn + &format!("{strings}\n");
    }
    Ok(strreturn)
}

fn pwd() -> Result<String, String> {
    match env::current_dir() {
        Ok(d) => Ok(d.display().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn remove_f_d(arguments: String) -> Result<(), String> {
    // Split_whitespace do what name says
    let mut b_arguments: Vec<&str> = arguments.split_whitespace().collect();
    // Buff to store the position of argument for recursive
    let mut b_argspo = 2023;

    // As is a Vec<str> check if some of they is "-r"

    let recursive: bool = if b_arguments.contains(&"-r") {
        // iterate over the vector and position method returns the position of
        // element if match internal condition
        b_argspo = match b_arguments.iter().position(|e| e == &"-r"){
            Some(d) => d,
            None => return Err(format!("Error getting position of -r argument")),
        };
        true
    } else {
        false
    };

    if b_argspo != 2023 {
        b_arguments.remove(b_argspo);
    }

    let mut a_files: Vec<&str> = Vec::new();
    let mut a_dirs: Vec<&str> = Vec::new();

    for i in &b_arguments {
        if Path::new(i).is_dir() {
            a_dirs.push(i);
        } else {
            a_files.push(i);
        }
    }

    if recursive {
        for d in a_dirs {
            match std::fs::remove_dir_all(d) {
                Ok(_d) => (),
                Err(_e) => eprintln!("Error deleting directory; {}, verify if exists and if you have right permissions.", d),
            }
        }
        for d in a_files {
            match std::fs::remove_file(d) {
                Ok(_d) => (),
                Err(_e) => eprintln!(
                    "Error deleting file; {}, verify if exists, if you have right permissions",
                    d
                ),
            }
        }
    } else {
        for d in a_dirs {
            match std::fs::remove_dir(d) {
                Ok(_d) => (),
                Err(_e) => eprintln!("Error deleting directory; {}, verify if exists, if you have right permissions and if is not empty (in which you need use -r for it)", d),
            }
        }
        for d in a_files {
            match std::fs::remove_file(d) {
                Ok(_d) => (),
                Err(_e) => eprintln!(
                    "Error deleting file; {}, verify if exists, if you have right permissions",
                    d
                ),
            }
        }
    }
    Ok(())
}

fn show(input: &String) -> Option<String> {
    let arguments: Vec<String> = input.trim().split(' ').map(|e| e.to_string()).collect();
    //let string_return = String::new();

    // Init the configuration as clean
    let mut config = libconfarg::ShowConfiguration {
        clean: false,
        stdin: false,
        hexa: false,
    };

    if arguments.checkarguments_help("show") {
        return None;
    }

    // The vec<String> return with files index is stored in "archives" variable.
    // The method is from RavnArguments trait.
    let mut options: Vec<&str> = Vec::new();
    let archives: Vec<String> = arguments.check_arguments("show", &mut options);

    if archives.len() == 0 || archives[0] == "" {
        return None;
    }

    for confs in options {
        if confs == "clean" {
            config.clean = true;
        } else if confs == "stdin" {
            config.stdin = true;
        } else if confs == "hexa" {
            config.hexa = true;
        }
    }

    // Stdinput
    if config.stdin {
        // Buffer variable to store returns
        let mut buffer = String::new();
        // To work with stdin
        // Match takes the read_line output, if is
        // Ok(_i) will print the buffer variable, but if
        // is Err(j) will print "j" (the error per se).
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_i) => {
                let buff = format!("stdin {{ {buffer} }}");
                return Some(buff);
            }
            Err(_j) => return None,
        }
    }

    // Opening files and showing them
    //let mut fstring: Vec<String> = Vec::new();
    let mut buffer: String = String::new();
    let mut fvec = HashMap::new();

    for names in &archives {
        if archives.len() == 1 {
            buffer = String::from_utf8_lossy( match &fs::read(names){
                Ok(d) => d,
                Err(_e) => return None,
            }).to_string();

            if !config.clean && !config.hexa {
                buffer = format!("{names} {{ {buffer} }}");
            } else if config.clean && !config.hexa {
                {}
            }

            if config.hexa {
                // Hexa mode
                // Remember; each char will be stored as hexa.
                let mut buffer2: String = String::new();
                // Split the file's data in lines() and collect each in &str vector.
                for iteration in buffer.lines().collect::<Vec<&str>>() {
                    // Splits each line in chars
                    for dchar in iteration.chars() {
                        // Transform each char in string and then into bytes data
                        for fchar in dchar.to_string().into_bytes() {
                            // Show each byte char into hexadecimal mode.
                            buffer2 += &(format!("{:x} ", fchar)).to_string();
                        }
                    }
                }
                buffer = buffer2;
                // End; archives.len() == 1
            }

            if config.clean && config.hexa {
                {}
            } else if !config.clean && config.hexa {
                buffer = format!("{} {{ {} }}",names,buffer);
            }
    } else {
        let mut buffer_hexa: String = String::new();
        let buffer_string: String = String::from_utf8_lossy( match &fs::read(names){
            Ok(d) => d,
            Err(_e) => return None,
        }).to_string();

        if config.hexa {
            // Hexa mode
            // Remember; each char will be stored as hexa.
            // Split the file's data in lines() and collect each in &str vector.
            for iteration in buffer_string.lines().collect::<Vec<&str>>() {
                // Splits each line in chars
                for dchar in iteration.chars() {
                    // Transform each char in string and then into bytes data
                    for fchar in dchar.to_string().into_bytes() {
                        // Show each byte char into hexadecimal mode.
                        buffer_hexa += &(format!("{:x} ", fchar)).to_string();
                    }
                }
            }
            // I must use ".to_string()" into "names" variable because her type is &String
            // and I want just String
            fvec.insert(names.to_string(),buffer_hexa.clone());
        // End; archives.len() == 1
        } else {
            fvec.insert(names.to_string(),buffer_string);
        }

    }
    }

    let mut fstrings = String::new();
    if archives.len() == 1 {
        return Some(format!("{}", buffer));
    } else {
        for (names,files) in &fvec {
            if !config.clean {
                fstrings += &format!("{names} {{ {files} }}\n\n");
            } else {
                fstrings += &format!("{files}");
            }
        }
    }
    return Some(fstrings);
}

fn sleep(input: &String) -> Result<(), String> {
    let seconds = match (input.split(':').collect::<Vec<&str>>())[0].parse::<u64>(){
        Ok(d) => d,
        Err(_e) => 0,
    };
    let nanoseconds = match (input.split(':').collect::<Vec<&str>>())[1].parse::<u32>(){
        Ok(d) => d,
        Err(_e) => 0,
    };
    let finalcount = Duration::new(seconds, nanoseconds);
    thread::sleep(finalcount);
    Ok(())
}

fn seq(input: &String) -> Result<(), String> {
    let first = match (input.split(':').collect::<Vec<&str>>())[0].parse::<u64>(){
        Ok(d) => d,
        Err(_e) => 0,
    };
    let last = match (input.split(':').collect::<Vec<&str>>())[1].parse::<u64>(){
        Ok(d) => d,
        Err(_e) => 0,
    };
    let increment = match (input.split(':').collect::<Vec<&str>>())[2].parse::<u64>(){
        Ok(d) => d,
        Err(_e) => 0,
    };

    let mut count = first;
    while count <= last {
        println!("{count}");
        count += increment;
    }
    Ok(())
}

fn tail(input: &String) -> Result<(), &str> {
    if input.len() <= 1 {
        eprintln!(
            "Not enough arguments; _tail [number] [file] : show the last [number] lines of [file]."
        );
        return Err(" ");
    }
    let mut lnumber = match (input.split(' ').collect::<Vec<&str>>())[0].parse::<u64>(){
        Ok(d) => d,
        Err(_e) => return Err("Error getting line number"),
    };
    let vfile = match File::open(Path::new((input.split(' ').collect::<Vec<&str>>())[1])){
        Ok(d) => d,
        Err(_e) => return Err("Error getting file information"),
    };

    let mut buffer = BufReader::new(&vfile);
    let mut fdata = String::new();

    let _ = buffer.read_to_string(&mut fdata);

    let lines_vfile: Vec<&str> = fdata.lines().collect();
    let mut lines_file: usize = fdata.lines().count() - 1;

    drop(buffer);
    let mut buffer: Vec<&str> = Vec::new();

    while lnumber > 0 {
        buffer.push(lines_vfile[lines_file]);
        lines_file -= 1;
        lnumber -= 1;
    }

    for line in buffer.iter().rev() {
        println!("{}", line);
    }

    Ok(())
}

fn fwhich(input: &String) -> Result<String, &str> {
    let result: Vec<String> = which(input.to_string());
    let mut sreturn = String::new();
    if !result.is_empty() {
        if result.len() > 1 {
            for i in result {
                if !sreturn.is_empty() {
                    sreturn = sreturn + &"," + &i;
                } else {
                    sreturn = i;
                }
            }
        } else {
            for i in result {
                sreturn = i;
            }
        }
        return Ok(sreturn);
    } else {
        return Err("Not found");
    }
}

////////////////

// Check the builtin executing it
// The first argument is the command, the second the lbuilts list
// Returns Ok(d) with the stdout of builtin or Err(e) if doesn't match
pub fn rbuiltins(command: &str, b_arguments: String) -> Result<String, &str> {
    use self::*;

    let result: String;
    // We MUST use trim() because always there are some unwelcomme characters at the start/end
    let command = command.trim();

    if LBUILTINS.contains(&command) {
        if command == "basename" {
            match basename(&b_arguments) {
                Some(d) => Ok(d),
                None => Err("no file name"),
            }
        } else if command == "base64" {
            match base64(&b_arguments) {
                Some(d) => Ok(d),
                None => Err("no file or error opening"),
            }
        } else if command == "count" {
            result = count(&b_arguments);
            Ok(result)
        } else if command == "info" {
            // "self" is needed because this is a module, not a binary
            result = info();
            Ok(result)
        } else if command == "date" {
            date()
        } else if command == "decodebase64" {
            match decodebase64(&b_arguments) {
                Some(d) => Ok(d),
                None => Err("error converting"),
            }
        } else if command == "du" {
            match disk_usage(&b_arguments) {
                Some(d) => Ok(d),
                None => Err("Error reading path"),
            }
        } else if command == "echoraw" {
            result = echoraw(&b_arguments);
            Ok(result)
        } else if command == "expand" {
            result = expand(b_arguments);
            Ok(result)
        } else if command == "head" {
            let _ = head(&b_arguments);
            Ok("".to_string())
        } else if command == "home" {
            envhome()
        } else if command == "mkdir" {
            match mkdir_r(Path::new(&b_arguments)) {
                Ok(_d) => Ok("".to_string()),
                Err(_e) => {
                    if _e == "Help;\n mkdir [directory]" {
                        let error = "Help;\n mkdir [directory]";
                        Err(error)
                    } else {
                        Err("Error creating directory")
                    }
                }
            }
        } else if command == "mkfile" {
            match mkfile ( Path::new( &b_arguments ) ){
                Ok(_d) => Ok( "".to_string() ),
                Err(e) => {
                    if e == "Fail to create file, verify if path exists and if you have the right permissions." {
                        Err("Fail to create file, verify if path exists and if you have the right permissions.")
                    } else {
                        Err("File already exists.")
                    }
                },
            }
        } else if command == "move" {
            let _ = fdmove(b_arguments);
            Ok(" ".to_string())
        } else if command == "nl" {
            match number_line(&b_arguments) {
                Ok(()) => Ok("".to_string()),
                Err(_e) => Err("Error opening file, check permissions and file system"),
            }
        } else if command == "rm" {
            match remove_f_d(b_arguments) {
                Ok(()) => Ok("".to_string()),
                Err(_e) => Err("Error deleting object"),
            }
        } else if command == "proc" {
            match proc() {
                Ok(d) => Ok(d),
                Err(_e) => Err("Error getting processes"),
            }
        } else if command == "pwd" {
            match pwd() {
                Ok(d) => Ok(d),
                Err(_e) => Err("Error getting actual working directory"),
            }
        } else if command == "cd" {
            let _ = cd(b_arguments);
            Ok(" ".to_string())
        } else if command == "cp" {
            let buff = b_arguments.split(' ').collect::<Vec<&str>>();
            if buff.len() < 2 {
                let str: &str = "Very few arguments; [SOURCE] [DESTINATION]";
                return Err(str);
            }

            let source = buff[0];
            let destination = buff[1];

            drop(buff);
            match copy(Path::new(source), Path::new(destination)) {
                Ok(_d) => Ok("".to_string()),
                Err(e) => Err(e),
            }
        } else if command == "env" {
            result = environmentvar();
            Ok(result)
        } else if command == "false" {
            let fresult = ffalse(&b_arguments);
            // Why not use "if let" inside "if/else" ?
            // Well; https://github.com/rust-lang/rust/issues/53667
            if let Ok(()) = fresult {
                return Err("");
            } else if let Err(false) = fresult {
                return Ok("".to_string());
            }
            Ok("".to_string())
        } else if command == "id" {
            match id(&b_arguments) {
                Ok(d) => Ok(d),
                Err(_e) => Err("Error getting environment variables"),
            }
        } else if command == "join" {
            match join(&b_arguments) {
                Ok(()) => Ok("Joined files".to_string()),
                Err(_e) => Err("Error joining files, verify arguments, permissions and/or space"),
            }
        } else if command == "list" {
            result = format!(
                " Bultins (they are called with '_'); {{\n {:?}\n}}",
                LBUILTINS
            );
            Ok(result)
        } else if command == "help" {
            result = format!("{HBUILTINS}");
            Ok(result)
        } else if command == "ln" {
            let buff = b_arguments.split(' ').collect::<Vec<&str>>();
            if buff.len() < 2 {
                let str: &str = "Very few arguments; [SOURCE] [DESTINATION]";
                return Err(str);
            }

            let source = buff[0];
            let destination = buff[1];

            drop(buff);
            match ln(Path::new(source), Path::new(destination)) {
                Ok(d) => Ok(d),
                Err(_e) => Err("Error creating symlink, maybe destionation already exists"),
            }
        } else if command == "ls" {
            result = ls(&b_arguments);
            Ok(result)
        } else if command == "show" {
            match show(&b_arguments) {
                Some(d) => Ok(d),
                None => Err(""),
            }
        } else if command == "sleep" {
            let _ = sleep(&b_arguments);
            Ok("".to_string())
        } else if command == "seq" {
            let _ = seq(&b_arguments);
            Ok("".to_string())
        } else if command == "tail" {
            let _ = tail(&b_arguments);
            Ok("".to_string())
        } else if command == "which" {
            match fwhich(&b_arguments) {
                Ok(d) => Ok(d),
                Err(_e) => Err("Not found"),
            }
        } else if command == "clear" {
            let _ = clear();
            Ok(" ".to_string())
        } else {
            Err("builtin not recognized")
        }
    } else {
        Err("not builtin found")
    }
}
