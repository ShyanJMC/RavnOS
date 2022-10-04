// Standar libraries
// Input Output lib
use std::io;
// Environment lib
use std::env;
// File System lib
use std::fs;
// Path lib
use std::path::{Path, PathBuf};
//  Metadata lib
use std::os::unix::fs::MetadataExt;
// Process lib
use std::process;

// RavnOS libraries
use libconfarg::RavnArguments;
use libstream::getprocs;

// Take as input the directory's name . We use "&" in String because
// in a loop the argument is always passed by reference.
fn readdir(input: &String) -> Vec<PathBuf> {
    // Read the directory
    let entries = fs::read_dir(input)
        .unwrap()
        // Take the "DirEntry" struct from "read_dir" and returns the full path
        .map(|res| res.map(|e| e.path()))
        // Here we customice the collect method to returns as Result<V,E>
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    entries
}

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

    if arguments.checkarguments_help("ls".to_string()) {
        process::exit(0);
    }

    // The vec<String> return with files index is stored in "lists" variable.
    // The method is from RavnArguments trait.
    let lists: Vec<String> = arguments.checkarguments_ls(&mut config);

    if config.proc {
        getprocs();
    }

    for names in &lists {
        let mut entries = Vec::new();
        let mut fbuffer = String::new();
        // Check if arguments is directory.
        if Path::new(names).is_dir() {
            entries = readdir(names);
            // I must use here and not as "readdir" method because if not the type will be forced to
            // "()" (which is more like a unit and at the same time is a type also).
            entries.sort();
        } else {
            if Path::new(names).is_file() {
                fbuffer = String::from(names);
                entries.push(PathBuf::from(fbuffer));
            } else {
                eprintln!("{names}; no such file or directory.");
                process::exit(1);
            }
        }

        if config.lines && !config.clean {
            println!("\nList of elements in {}; {}", names, &entries.len());
        }

        if config.verbose && !config.clean {
            // Here I found an issue; as "readdir" returns Vec<PathBuf>, "metadata" will have
            // issues in some paths like; ~/ . Because of that, we must convert it to String. 
            for h in &entries {
                let vartemp: Vec<String> = h.clone().into_os_string().into_string().unwrap().lines().map(|e| e.to_string()).collect();
                for ff in vartemp {
                    buffer.push(format!("{} {}", &ff, fs::metadata( &ff ).unwrap().size() ));
                }
            }
            for ee in &buffer {
                println!("{}", ee);
            }
        } else {
            if !config.clean {
                println!("{names};\n{:?}", &entries);
            } else {
                println!("{:?}", &entries);
            }
        }

        // I must do this because if I use "clean()" will be dropped from memory.
        buffer = Vec::new();
    }
}
