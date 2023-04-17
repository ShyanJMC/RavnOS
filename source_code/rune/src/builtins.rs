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

// Read lib
use std::io::Read;

use libstream::Stream;
use libstream::search_replace_string;

// Here we use a const and not let because is a global variable
// As we know the size of each word we can use "&str" and then we specify the number
// of elements. This is because a const must have know size at compiling time.
const LBUILTINS: [&str; 16] = ["cd", "clear", "cp", "disable_history", "enable_history", "env", "exit", "history", "home", "info", "mkdir", "mkfile", "list", "pwd", "rm", "$?"];
const RUNE_VERSION: &str = "v0.15.3";

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

fn cd(path: String) -> () {
    if path.is_empty(){
        eprintln!("Help;\n cd [PATH]");
    } else {
        let buff = path.trim();
        let npath = Path::new( &buff );
        env::set_current_dir(&npath).expect("Failing setting the new working path");
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

        println!("sd; {sd}");

        // Destionation directory
        let dd: String = {
            let temp = dest.display().to_string();
            let buffer: Vec<&str> = temp.split('/').collect();
            // For absolute path
            if buffer.len() > 1 {
            	format!("{}", &buffer[buffer.len()-1] )
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

        println!("dd; {dd}");


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

                println!("npath file; {:?}",npath);

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
        if command == "info" {
            // "self" is needed because this is a module, not a binary
            result = info();
            Ok(result)
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
        } else if command == "list" {
            result = format!(" Bultins, they are called with '_'; {{\n {:?}\n}}", LBUILTINS);
            Ok(result)
        } else if command == "clear" {
            clear();
            Ok( " ".to_string() )
        } else {
            Err( "builtin not recognized" )
        }
    } else {
        Err( "not builtin found" )
    }
}
