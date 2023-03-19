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

// Standar libraries
// Environment lib
use std::env;
// File System lib
use std::fs;
// Path lib
use std::path::{Path, PathBuf};
//  Metadata lib
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

// Process lib
use std::process::{self, Command};

// RavnOS libraries
extern crate libconfarg;
extern crate libstream;
extern crate libfile;

use libconfarg::RavnArguments;
use libstream::{getprocs, Stream, Epoch};
use libfile::RavnSizeFile;

fn main() {
    // env::args() takes program's arguments
    // collect() takes arguments and returns in tuple
    let mut arguments: Vec<String> = env::args().collect();

    // Remove the binary name from vector (remove the data from index zero).
    arguments.remove(0);

    // Result buffer
    let mut buffer: Vec<String> = Vec::new();

    // Init the configuration as clean
    let mut config = libconfarg::LsConfiguration {
        verbose: false,
        proc: false,
        lines: false,
        clean: false,
    };

    if arguments.checkarguments_help("ls") {
        process::exit(0);
    }

    // The vec<String> return with files index is stored in "lists" variable.
    // The method is from RavnArguments trait.
    let mut options: Vec<&str> = Vec::new();
    let mut lists: Vec<String> = arguments.check_arguments("ls", &mut options);

    if lists.is_empty(){
    	lists.push(env::current_dir().unwrap().display().to_string());
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
        println!("{strings}");
        }
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
            		},
            	false => {
            		// Check if arguments is a file.
            		if Path::new(names).is_file() {
                		// If is a file, store it in variables.
                		entries.push(PathBuf::from(names));
            		}
            	}
        }


        if config.lines && !config.clean {
            println!("\nList of elements in {}; {}", names, &entries.len());
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
					eprintln!("{}: File/dir/symlink do not exist, is invalid or is broken.", h.display());
					// break the loop for current stage
					continue;
				}

                let fmetadata = fs::metadata(h.display().to_string()).unwrap();


                // ID numeric to user
                let ownerout = Command::new("/usr/bin/id")
                    .arg(fmetadata.uid().to_string())
                    .output()
                    .unwrap();
                // When you use "output" method, the stdout of command will be stored in
                // "stdout" field. But, is stored as u8, and needs to be processed as utf8.

                //let owner = std::str::from_utf8(&ownerout.stdout)
                //    .unwrap()
                //    .strip_suffix("\n")
                //    .unwrap();

                let owner = match std::str::from_utf8(&ownerout.stdout) {
                	Err(_e) => "Error reading owner, check file/dir permissions.",
                	Ok(d) => match d.strip_suffix('\n') {
                		Some(d) => d,
                		None => "Error reading owner, check file/dir permissions.",
                	},
                };

                buffer.push(format!(
                    "{} {} {:?} {} {}",
                    match &h.is_file() {
                    	true => format!("f: {}", h.display() ),
                    	false => format!("d: {}", h.display() ),
                    },
                    fmetadata.mtime().epoch_to_human(),
                    // Permissions
                    // Permissions method by default will return in bits, if you want the octal chmod
                    // syntax need to use ".mode()".
                    // As Octal is not a type by it self, we need use "format!" macro to convert it in
                    // octal mode, the return is a String.
                    format!("{:o}", fmetadata.permissions().mode()).permission_to_human(),
                    owner,
                    fmetadata.size().size_to_human()
                ));
            }
            // Show filename and size.
            for ee in &buffer {
                println!("{}", ee);
            }
        } else {
            if !config.clean && lists.len() > 1 {
                println!("{names};\n{:?}", &entries);
            } else {
                println!("{:?}", &entries);
            }
        }

        // I must do this because if I use "clean()" will be dropped from memory.
        buffer = Vec::new();
    }
}
