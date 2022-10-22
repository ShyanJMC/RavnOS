//! This file is part of RavnOS.
//!
//! RavnOS is free software: 
//! you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, 
//! either version 3 of the License, or (at your option) any later version.
//!
//! RavnOS is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; 
//! without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License along with RavnOS. If not, see <https://www.gnu.org/licenses/>

//!
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022

// Standar libraries

//// Environment lib
use std::env;
//// Filesystem lib
use std::fs;
//// String slice lib
// use std::str;
//// For Unix platform
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

//// I/O lib
use std::io;
// Process lib
use std::process::{self, Command};

// RavnOS libraries
use libconfarg::RavnArguments;
use libstream::{getprocs, Stream};
use libfile::RavnFile;

fn main() {
    // env::args() takes program's arguments (the first is always the self binary).
    // collect() takes arguments and returns in tuple
    let mut arguments: Vec<String> = env::args().collect();

    // Remove the binary name from vector (remove the data from index zero).
    arguments.remove(0);

    // Init the configuration as clean
    let mut config = libconfarg::ShowConfiguration {
        size: false,
        datetime: false,
        lines: false,
        owner: false,
        permission: false,
        clean: false,
        stdin: false,
        proc: false,
        hexa: false,
        words: false,
    };

    if arguments.checkarguments_help("show") {
        process::exit(0);
    }

    // The vec<String> return with files index is stored in "archives" variable.
    // The method is from RavnArguments trait.
   let mut options: Vec<&str> = Vec::new();
    let archives: Vec<String> = arguments.check_arguments("show",&mut options);

    for confs in options {
        if confs == "size" {
            config.size = true;
        } else if confs == "datetime" {
            config.datetime = true;
        } else if confs == "lines" {
            config.lines = true;
        } else if confs == "owner" {
            config.owner = true;
        } else if confs == "permission" {
            config.permission = true;
        } else if confs == "clean" {
            config.clean = true;
        } else if confs == "stdin" {
            config.stdin = true;
        } else if confs == "proc" {
            config.proc = true;
        } else if confs == "hexa" {
            config.hexa = true;
        } else if confs == "words" {
            config.words = true;
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
        match io::stdin().read_line(&mut buffer) {
            Ok(_i) => println!("{buffer}"),
            Err(j) => println!("error; {j}"),
        }
    }

    // Showing procs
    if config.proc {
        getprocs();
    }

    // Opening files and showing them
    for names in &archives {
        let meta = fs::metadata(&names).unwrap();
        // " if X = false " is equal to; " if !X "
        // " if X = true " is equal to; " if X "
        if !config.clean {
            if archives.len() > 1 {
                println!("File Name: {names}");
            }
            if config.size {
                println!("Size: {}", meta.size().size_to_human());
            }

            if config.datetime {
                println!(
                    "Modified (EPoch): {:?}\nAccessed (EPoch): {:?}\nCreated (EPoch): {:?}",
                    meta.mtime(),
                    meta.atime(),
                    meta.ctime()
                );
            }

            if config.lines {
                println!(
                    "Lines: {:?}",
                    fs::read_to_string(names)
                        .expect("Error reading file.")
                        .lines()
                        .count()
                );
            }

            if config.words {
                println!(
                    "Words - Letters; {:?}",
                    fs::read_to_string(names).expect("Error reading file").word_count()
                    );
            }

            if config.owner {
                // Check if the target OS is some of Unix family.
                if cfg!(target_os = "linux")
                    || cfg!(target_os = "freebsd")
                    || cfg!(target_os = "dragonfly")
                    || cfg!(target_os = "openbsd")
                    || cfg!(target_os = "netbsd")
                    || cfg!(target_family = "unix")
                {
                    // Call "id" command to detect who is ID number owner.
                    let ownerout = Command::new("/usr/bin/id")
                        .arg(meta.uid().to_string())
                        .output()
                        .unwrap();
                    // When you use "output" method, the stdout of command will be stored in
                    // "stdout" field. But, is stored as u8, and needs to be processed as utf8.
                    println!(
                        "Owner: {}",
                        std::str::from_utf8(&ownerout.stdout)
                            .unwrap()
                            .strip_suffix("\n")
                            .unwrap()
                    );
                } else {
                    println!("Owner: Not supported because platform is not detected as Unix.");
                }
            }

            if config.permission {
                // Permissions method by default will return in bits, if you want the octal chmod
                // syntax need to use ".mode()".
                // As Octal is not a type by it self, we need use "format!" macro to convert it in
                // octal mode, the return is a String.
                println!(
                    "Permission: {:?}",
                    format!("{:o}", meta.permissions().mode()).permission_to_human()
                );
            }
        }

        // Opening file and reading it as string.
        let fstring = fs::read_to_string(names).expect("Error reading file.");
        // Using by reference to not take "len" the ownership of "archives".

        if !config.clean && archives.len() > 1 {
            println!("=================\n");
        }
        if archives.len() > 1 && !config.proc {
            println!(
                "{}\n\n-------------------------------------------------\n",
                fstring
            );
        } else {
            if !config.hexa {
                println!("{}", fstring);
            }
            else {
                // Hexa mode
                // Remember; each char will be printed as octal.
                // Split the file's data in lines() and collect each in &str vector.
                for iteration in fstring.lines().collect::<Vec<&str>>() {
                    // Splits each line in chars
                    for dchar in iteration.chars() {
                        // Transform each char in string and then into bytes data
                        for fchar in dchar.to_string().into_bytes() {
                            // Show each byte char into hexadecimal mode.
                            print!("{:x} ", fchar );
                        }
                    }
                }
            }
        }
    }
}
