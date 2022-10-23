// Standard libraries

// Environment lib
use std::env;
// Process lib
use std::process;

// RavnOS libraries
use libconfarg::RavnArguments;
use libstream::{getprocs, file_filter, Stream};

fn main() {
    // env::args() takes program's arguments (the first is always the self binary).
    // collect() takes arguments and returns in tuple
    let mut arguments: Vec<String> = env::args().collect();

    // Remove the binary name from vector (remove the data from index zero).
    arguments.remove(0);

	// Check if some arguments are help
    if arguments.checkarguments_help("search"){
        process::exit(0);
    }

	// Configuration struct
    let mut inst1 = libconfarg::SearchConfiguration {
        file: false,
        directory: false,
        environment: false,
        processes: false,
    };

	// Vector to store options
	let mut options: Vec<&str> = Vec::new();
	let mut inputs: Vec<String> = arguments.check_arguments("search", &mut options);
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
			for outputs in file_filter( &files, ssearch.clone() ) {
				if inputs.len() > 1 {
					println!("{files}: {outputs}\n" );
				} else {
					println!("{outputs}");
				}
			}
		}
	}

	if inst1.environment{
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
			let output = files.readdir();
			for archives in output {
				if archives.to_str().unwrap().contains(&ssearch) {
					println!("{:?}",archives);
				}
			}
		}
	}
}
