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
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022

// Standard libraries

// Environment lib
use std::env;
// Process lib
use std::process;

// Stdinput and stdoutput read
use std::io::Read;

// RavnOS libraries
extern crate libconfarg;
extern crate libstream;

use libconfarg::RavnArguments;
use libstream::{file_filter, getprocs, Stream};

fn main() {
    // env::args() takes program's arguments (the first is always the self binary).
    // collect() takes arguments and returns in tuple
    let mut arguments: Vec<String> = env::args().collect();

    // Remove the binary name from vector (remove the data from index zero).
    arguments.remove(0);

    // Check if some arguments are help
    if arguments.checkarguments_help("search") {
        process::exit(0);
    }

    // Configuration struct
    let mut inst1 = libconfarg::SearchConfiguration {
        file: false,
        directory: false,
        environment: false,
        processes: false,
        recursive: false,
        input: false,
        ravnkey: false,
    };

    // Vector to store options
    let mut options: Vec<&str> = Vec::new();
    let mut inputs: Vec<String> = arguments.check_arguments("search", &mut options);
    if inputs.is_empty() {
        process::exit(0);
    }
    // String to search must be the first position (0)
    let ssearch: String = inputs[0].clone();
    // Delete the string to search from the vector because can be more than just 1 file in which search.
    inputs.remove(0);

    // Check arguments
    for confs in options {
        if confs == "file" {
            inst1.file = true;
        } else if confs == "directory" {
            inst1.directory = true;
        } else if confs == "environment" {
            inst1.environment = true;
        } else if confs == "proc" {
            inst1.processes = true;
        } else if confs == "recursive" {
            inst1.recursive = true;
        } else if confs == "input" {
            inst1.input = true;
        } else if confs == "ravnkey" {
            inst1.ravnkey = true;
        }
    }

    // Search for [key] and extract [data]
    if inst1.ravnkey {
        let mut stdin_buffer: Vec<u8> = Vec::new();
        let stdinvar = std::io::stdin();
        let mut locking = stdinvar.lock();

        // Shadowing
        let ssearch = ssearch.as_str();

        locking
            .read_to_end(&mut stdin_buffer)
            .expect("Error reading stdin.");
        let string: &str =
            std::str::from_utf8(&stdin_buffer).expect("Error converting input to UTF-8 strings.");
        let hmap = string.to_string().readkey();

        match hmap.get(ssearch) {
            Some(e) => println!("{e}"),
            None => eprintln!("Not found"),
        }
    }

    // Search in stdin
    if inst1.input {
        // Where to save data
        let mut stdin_buffer: Vec<u8> = Vec::new();
        // Stdinput control
        let stdinvar = std::io::stdin();
        // Block stdinput to control it
        let mut locking = stdinvar.lock();

        // Take in consideration this; read_to_end saves data in u8
        locking
            .read_to_end(&mut stdin_buffer)
            .expect("Error reading stdin.");

        // Transform u8 (integers 8 bytes) to strings
        let strings =
            std::str::from_utf8(&stdin_buffer).expect("Error converting input to UTF-8 strings.");

        // Read stdin_buffer and search in each line
        for inl in strings.lines() {
            if inl.contains(&ssearch) {
                println!("{inl}");
            }
        }
    }

    // Search recursively
    if inst1.recursive {
        let results = inputs[0].readdir_recursive();
        // Each struct field is Vec<String> so, we must use an iterator over them

        // Dir
        for dir in &results.dbuff {
            if dir.contains(&ssearch) {
                println!("d; {dir}");
            }
        }
        // Files
        for files in &results.fbuff {
            for outputs in file_filter(&files, ssearch.clone()) {
                println!("f; {files}\n\n {outputs}\n\n");
            }
        }
    }

    // Start to check and work.
    // Remember; if XX {YYY} will execute YYYY if XX's returns is true.
    if inst1.processes {
        // Get procs and store it
        let buffer1: Vec<String> = getprocs();
        // Iterate over each string
        for procs in buffer1 {
            // Check if that string contains the search string
            if procs.contains(&ssearch) {
                println!("{procs}");
            }
        }
    }

    if inst1.file {
        for files in &inputs {
            for outputs in file_filter(&files, ssearch.clone()) {
                if inputs.len() > 1 {
                    println!("{files}: {outputs}\n");
                } else {
                    println!("{outputs}");
                }
            }
        }
    }

    if inst1.environment {
        for strings in &inputs {
            let data = match env::var(strings) {
                Ok(value) => value,
                Err(_e) => "".to_string(),
            };
            if data.contains(&ssearch) {
                println!("{strings}: {data}");
            }
        }
    }

    // I know, is not a fully recursive, will be a future for the
    // release stable version
    if inst1.directory {
        for files in &inputs {
            let mut output = files.readdir();
            output.sort();
            for archives in output {
                if archives.to_str().unwrap().contains(&ssearch) {
                    println!("{:?}", archives);
                }
            }
        }
    }
}
