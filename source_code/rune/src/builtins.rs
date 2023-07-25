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
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022-2023

// Environment lib
use std::env;

// Path lib
use std::path::Path;

// Filesystem lib
use std::fs::{self,File};

// IO lib
use std::io::{self, Read, Write};
use std::io::BufReader;

// Unix lib
use std::os::unix::fs::symlink;

use libstream::Stream;
use libstream::search_replace_string;
use libstream::file_filter;



// Because this file is not a binary or lib, is just another module, to import
// under score another module we must use "crate"
use crate::io_mods::get_user_home;


// Here we use a const and not let because is a global variable
// As we know the size of each word we can use "&str" and then we specify the number
// of elements. This is because a const must have know size at compiling time.
const LBUILTINS: [&str; 24] = ["basename", "cd", "clear", "cp", "disable_history", "echo_raw", "enable_history", "env", "exit", "expand", "history", "head", "home", "id", "join", "info", "mkdir", "mkfile", "move", "list", "ln", "pwd", "rm", "$?"];

const HBUILTINS: &str = "Help;
Remember respect the positions of each argument

_basename: takes a path and prints the last filename
_cd [PATH]: If path do not exist, goes to user home directory
_clear: Clean the screen
_cp [source] [destination]: copy file or directory from [source] to [destination]
_disable_history: disable save commands to history without truncate the file
_enable_history: enable save commands to history
_echo_raw: show string into stdout without interpreting special characters
_env: show environment variables
_exit: exit the shell properly
_expand: convert tabs to spaces in file (with new file; [FILE]-edited), with '-t X' you can specify the spaces number, first the options (if exists) and then the file.
_head -n [number] [file]: show [number] first lines for file.
_history: show the history commands with date and time
_home: returns the current user's home directory
_id [options]: show current user, '-n' for name and '-u' for UUID
_info: show system's information
_join [file_1] [file_n] [destination]: joins files into destionation file
_mkdir [dest] : create directory if it has more subdirectories it will create them recursively
_mkfile [file]: create empty file
_list: list builtins like this
_ln [source] [dest]: creates a link [dest] to [source]
_pwd: print the current directory
_rm [target]: delete the file or directory, if the directory have files inside must use '-r' argument to include them.
_$?: print the latest command exit return, not include builtins";

const RUNE_VERSION: &str = "v0.25.18";

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
        for (key, value) in std::env::vars(){
            if key == "USER" {
                varvalue = String::from( value);
                break;
            }
        }
        varvalue
    };

    format!(" RavnOS's Shell\n Copyright 2023 Joaquin 'ShyanJMC' Crespo\n Rune Shell version; {RUNE_VERSION}\n OS: {}\n User: {} ",os, user)
}

// Takes the path and returns only the file_name
fn basename(input: &String) -> Option<String> {
    // Path generation
    let _buff = Path::new(&input);
    // If "file_name" type returns is equal to Some(X) do that
    if let Some(filename) = _buff.file_name() {
        // To avoid use "unexpect" or "unwrap"
        return Some(match filename.to_str(){
            Some(d) => d,
            None => "",
        }.to_string());
    }
    None
}

fn environmentvar() -> String {
    let mut buffer: Vec<String> = Vec::new();
    let mut buffer2: String = String::new();
    for (key, value) in std::env::vars(){
        let envv = format!("{}; {}\n", key, value);
        buffer.push( envv );
    }
    for data in buffer {
        buffer2 = buffer2 + &data.to_string();
    }
    buffer2
}

fn expand(input: String) -> String {
    if input.contains("-t"){
        let s_number: usize;
        let args = input.split(' ').collect::<Vec<&str>>();
        if args[0] == "-t" {
            s_number = args[1].trim().parse().unwrap();
            let mut file = match File::open(args[2].clone()) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error opening file; {e}");
                    return String::from("Error opening file.");
                }
            };
            let mut string = String::new();
            file.read_to_string(&mut string);
            let ninput: String = match search_replace_string(&string,&'\t'.to_string(), &" ".repeat(s_number)){
                Ok(d) => d,
                Err(_e) => String::from("Matching not found"),
            };

            let nfile = args[2].clone().to_string() + "-edited";
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
        file.read_to_string(&mut string);
        let ninput: String = match search_replace_string(&string,&'\t'.to_string(), &"        ".to_string()){
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

fn cd(path: String) -> () {
    if path.is_empty(){
        // Goes to home user dir
        let binding = get_user_home();
    	let home: &str = binding.as_str().clone();
        match env::set_current_dir(&home) {
            Ok(d) => d,
            Err(_e) => {
                eprintln!("Fail changing to current home directory");
            },
        }
    } else {
        let buff = path.trim();
        let npath = Path::new( &buff );
        match env::set_current_dir(&npath) {
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

fn echo_raw(input: &String) -> String {

    input.clone()
}
// This function not copy directly using the kernel's filesystem
// to avoid any possible issue we copy bit a bit directly.
fn copy<'a>(source: &Path, dest: &Path) -> Result<(),&'a str> {
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

                match env::current_dir(){
            		Ok(d) => format!("{}/{}",d.display().to_string(),source.display().to_string()),
            		Err(e) => {
            			eprintln!("{e}");
            			return Err("Error getting current dir for source");
            		},
            	}

            }
        };

        // Destination directory
        let dd: String = {
            let temp = dest.display().to_string();
            let buffer: Vec<&str> = temp.split('/').collect();
            // For absolute path
            if buffer.len() > 1 {
            	format!("{}", dest.display().to_string() )
            }
            // For relative path
            else {
            	match env::current_dir(){
            		Ok(d) => format!("{}/{}",d.display().to_string(),buffer[0]),
            		Err(e) => {
            			eprintln!("{e}");
            			return Err("Error getting current dir for destionation");
            		},
            	}
           }
        };

        // To detect if "source" is a file
        if source.is_file(){

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
                Ok(_d) => {},
                Err(_e) => {
                    let str: &str = "Error reading source file, check permissions.";
                    return Err(str);
                }
            };

            // Create the file
            match File::create(dest.display().to_string()) {
                Ok(_d) => {},
                Err(_e) => {
                    print!("{_e}; {}",dest.display());
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

                drop(source);

                match mkdir_r( Path::new(&ddir) ){
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
                        let str: &str = "Error writting buffer to destination file, check permissions.";
                        return Err(str);
                    }
                }
            }
        }

        Ok(())


    }
}

fn head(input: &String){
    let mut file;
    let mut fdata = Default::default();
    let mut lnumber = 0;
    let mut cnumber = 1;
    let s_lnumber: Vec<&str> = input.split(' ').collect();
    if input.contains("-n") && s_lnumber.len() <=2 {
        eprintln!("Not enough arguments. Remember; head -n [number] [file]");
        return;
    }
    if input.contains("-n"){
        lnumber = match s_lnumber[1].trim().parse(){
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error parsing str to int; {e}\nVerify kernel compatiblity with Rust std");
                return;
            }
        };
    }
    file = match File::open(s_lnumber[2].trim()){
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error opening file; {e}");
                return;
            }
    };

    drop(s_lnumber);

    // Is more eficient that a raw read
    let mut buff = BufReader::new(&file);
    buff.read_to_string(&mut fdata);
    drop(file);
    for i in fdata.lines(){
        if cnumber <= lnumber {
            println!("{i}");
            cnumber += 1;
        } else {
            return;
        }
    }

}

fn id(input: &String) -> Result<String,String> {
    let mut buff = String::new();
    let username = match env::var("USERNAME") {
        Ok(d) => d,
        Err(e) => e.to_string(),
    };
    if input.contains("-n"){
        buff = username.to_string();
    }
    if input.contains("-u"){
        let uuid_line = file_filter(&"/etc/passwd".to_string(), username.clone());
        // The return is a Vec<&str> we take the [1] position
        let uuid = ( uuid_line[0].split(':').collect::<Vec<&str>>() )[2];
        let guid = ( uuid_line[0].split(':').collect::<Vec<&str>>() )[3];
        println!("userid {{ {uuid} }} ");
        println!("groupid {{ {guid} }} ");
    }

    if input.contains("-g"){
        let guid_line = file_filter(&"/etc/group".to_string(), username);
        let mut groups = String::new();
        for i in guid_line {
                let name = (i.split(':').collect::<Vec<&str>>())[0].clone();
                let id = (i.split(':').collect::<Vec<&str>>())[2].clone();
                groups = groups + &name + ":" + &id + "\n";
        }

        println!("groups; {{ {groups} }}");
    }

    if !input.contains("-u") && !input.contains("-n") && !input.contains("-g"){
        println!("Usage; '-n' for name, '-g' for user groups and '-u' for UUIDs");
    }
    Ok(buff)
}

fn join(input: &String) -> Result<(),&str> {
    if input.len() <= 2 {
        return Err("Not enough arguments");
    }
    let mut files: Vec<&str> = input.split_whitespace().collect();
    let lenght = files.len();
    let mut destination = match File::create(files[lenght-1]) {
        Ok(d) => d,
        Err(e) => {
            return Err("Error creating destination file");
        }
    };
    let mut fdata = String::new();
    files.remove(lenght-1);

    for i in files {
        let file = match File::open(i.trim()){
                Ok(d) => d,
                Err(e) => {
                    return Err("Error opening file");
                }
        };
        let mut buff = BufReader::new(&file);
        buff.read_to_string(&mut fdata);
        drop(file);

    }
    match destination.write_all(fdata.as_bytes()){
        Ok(d) => return Ok(()),
        Err(e) => {
            return Err("Error writting destination file from buffer");
        }
    };


}

fn ln(source: &Path, dest: &Path) -> Result<String,()>{
    match symlink(source, dest) {
        Ok(_d) => Ok( format!("Symlink created for {} pointing to {}", dest.display(), source.display() )),
        Err(_e) => Err(()),
    }
}

// Create a directory recusively
// We use "Path" type because it returns absolute path
fn mkdir_r(path: &Path) -> Result<u64,String> {
    if path.display().to_string().is_empty(){
        Err("Help;\n mkdir [directory]".to_string())
    } else {
        match std::fs::create_dir_all(path.display().to_string()){
            Ok(_d) => Ok(0),
            Err(e) =>  return Err( e.to_string() ),
        }
    }
}

fn mkfile(path: &Path) -> Result<(),&str> {
    if !path.exists() {
        match std::fs::File::create( path.display().to_string() ) {
            Ok(_d) => return Ok(()),
            Err(_e) => return Err ("Fail to create file, verify if path exists and if you have the right permissions."),
        }
    } else {
        Err("File already exists.")
    }
}

fn fdmove(input: String) {
    let arguments: Vec<_> = input.split(' ').collect();
    let n_arguments = arguments.len();
    let destination = arguments[n_arguments-1];
    let mut source: Vec<String> = Vec::new();

    for i in &arguments[0..(n_arguments-1)] {
        source.push(i.to_string());
    }

    if Path::new(destination).is_dir() {
        let mut ndestination: Vec<String> = Vec::new();
        for i in &source {
            let snumber: Vec<_> = i.split('/').collect();
            let sbuffer = snumber.len();
            ndestination.push(destination.to_string() + snumber.iter().nth(sbuffer-1).unwrap());
        }
        for i in &source {
            for j in &ndestination {
                match copy(Path::new(&i),Path::new(&j)){
                    Ok(_d) => {
                        remove_f_d(i.clone());
                        ()
                    }
                    Err(e) => eprintln!("{e}"),
                };

            }
        }

    } else {
        for i in &source {
            match copy(Path::new(&i),Path::new(&destination)){
                Ok(_d) => {
                    remove_f_d(i.to_string());
                    ()
                },
                Err(e) => eprintln!("{e}"),
            };

        }

    }
}

fn pwd() -> Result<String, String> {
	match env::current_dir() {
		Ok(d) => Ok( d.display().to_string() ),
		Err(e) => Err(e.to_string()),
	}
}

fn remove_f_d(arguments: String) -> Result<(),String> {
    // Split_whitespace do what name says
    let mut b_arguments: Vec<&str> = arguments.split_whitespace().collect();
    // Buff to store the position of argument for recursive
    let mut b_argspo = 2023;

    // As is a Vec<str> check if some of they is "-r"

    let recursive: bool = if b_arguments.contains(&"-r") {
            // iterate over the vector and position method returns the position of
            // element if match internal condition
            b_argspo = b_arguments.iter().position(|e| e == &"-r").unwrap();
            true
        } else {
            false
        };

    if b_argspo != 2023 {
        b_arguments.remove( b_argspo );
    }
    drop(b_argspo);

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
                Err(_e) => eprintln!("Error deleting file; {}, verify if exists, if you have right permissions", d),
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
                Err(_e) => eprintln!("Error deleting file; {}, verify if exists, if you have right permissions", d),
            }
        }
    }
    Ok(())
}

////////////////

// Check the builtin executing it
// The first argument is the command, the second the lbuilts list
// Returns Ok(d) with the stdout of builtin or Err(e) if doesn't match
pub fn rbuiltins(command: &str, b_arguments: String) -> Result<String,&str> {
    use self::*;

    let result: String;
    // We MUST use trim() because always there are some unwelcomme characters at the start/end
    let command = command.trim();

    if LBUILTINS.contains( &command ){
        if command == "basename" {
            match basename(&b_arguments){
                Some(d) => Ok(d),
                None => Err("no file name"),
            }

        } else if command == "info" {
            // "self" is needed because this is a module, not a binary
            result = info();
            Ok(result)
        } else if command == "echo_raw" {
            result = echo_raw(&b_arguments);
            Ok(result)
        }else if command == "expand" {
            result = expand(b_arguments);
            Ok(result)
        } else if command == "head" {
            head(&b_arguments);
            Ok("".to_string())
        } else if command == "mkdir" {
            match mkdir_r( Path::new( &b_arguments ) ) {
                Ok(_d) => Ok( "".to_string() ),
                Err(_e) => {
                    if _e == "Help;\n mkdir [directory]" {
                        let error = "Help;\n mkdir [directory]";
                        Err(error)
                    } else {
                        Err("Error creating directory")
                    }
                },
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
            fdmove(b_arguments);
            Ok(" ".to_string())
        } else if command == "rm" {
            match remove_f_d(b_arguments) {
                Ok(()) => Ok( "".to_string() ),
                Err(_e) => Err("Error deleting object"),
            }
        } else if command == "pwd" {
        	match pwd(){
        		Ok(d) => Ok(d),
        		Err(_e) => Err("Error getting actual working directory")
        	}
        } else if command == "cd" {
            cd(b_arguments);
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
            match copy( Path::new(source), Path::new(destination) ) {
                Ok(_d) => Ok( "".to_string() ),
                Err(e) => Err(e),
            }

        } else if command == "env" {
            result = environmentvar();
            Ok(result)
        } else if command == "id" {
            match id(&b_arguments) {
                Ok(d) => Ok(d),
                Err(e) => Err("Error getting environment variables"),
            }
        } else if command == "join" {
            match join(&b_arguments){
                Ok(()) => Ok("Joined files".to_string()),
                Err(e) => Err("Error joining files, verify arguments, permissions and/or space"),
            }
        } else if command == "list" {
            result = format!(" Bultins, they are called with '_'; {{\n {:?}\n}}\n\n{HBUILTINS}", LBUILTINS);
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
            match ln( Path::new(source), Path::new(destination) ) {
                Ok(d) => Ok(d),
                Err(_e) => Err("Error creating symlink, maybe destionation already exists"),
            }
        }else if command == "clear" {
            clear();
            Ok( " ".to_string() )
        } else {
            Err( "builtin not recognized" )
        }
    } else {
        Err( "not builtin found" )
    }
}
