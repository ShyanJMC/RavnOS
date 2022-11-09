//! This file is part of RavnOS.
//!
//! RavnOS is free software: 
//! you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, 
//! either version 3 of the License, or (at your option) any later version.
//!
//! RavnOS is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; 
//! without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License along with RavnOS. If not, see <https://www.gnu.org/licenses/>. 

//!
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022

//!
//! RavnOS's libstream
//! This file contains some modules to work with stream data like; word_count, readdir, permission_to_human, etc.


/// Filesystem System lib
use std::fs::{self, File};
/// Input Output lib
use std::io::{self, Read};
/// Standard path
use std::path::{PathBuf};

/// Struct for recursive reading
// With the derive(Clone) we allow it to be cloned
#[derive(Clone,Debug)]
pub struct DirStructure {
	pub dbuff: Vec<String>,
	pub fbuff: Vec<String>,
}

/// Outputs 
pub trait Stream {
    fn permission_to_human(&self) -> Vec<&'static str>;
    fn word_count(&self) -> Vec<usize>;
    fn readdir(&self) -> Vec<PathBuf>;
    fn readdir_recursive(&self) -> DirStructure;
}

impl Stream for String {
    /// Read directories and returns PathBuf with each file and directory.
    fn readdir(&self) -> Vec<PathBuf> {
        // Read the directory
        let entries = fs::read_dir(&self)
            .unwrap()
            // Take the "DirEntry" struct from "read_dir" and returns the full path
            .map(|res| res.map(|e| e.path()))
            // Here we customice the collect method to returns as Result<V,E>
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();

        entries
    }

    // Read dir recursive
    // Is not stable yet, I must fix first an issue with do not read sub dirs after first round
    fn readdir_recursive(&self) -> DirStructure { 
    	// I must use a closure here to not re write readdir function
    	let readdir = |path: String| -> Vec<PathBuf> {
    		// Read the directory
    		let entries = fs::read_dir(path).unwrap()
    		// Take the "DirEntry" struct from "read_dir" and returns the full path
    		.map(|res| res.map(|e| e.path()))
    		// Here we customice the collect method to returns as Result<V,E>
    		.collect::<Result<Vec<_>, io::Error>>()
    		.unwrap();

    		entries
    	};
    	
    	// Path buff
    	let mut vec: Vec<PathBuf> = readdir( self.clone() );

		// Structure
		let mut dstructure_complete = DirStructure {
			dbuff: Vec::new(),
			fbuff: Vec::new(),
		};

		let mut dstructure: Vec<String> = Vec::new();
		let mut fstructure: Vec<String> = Vec::new();
		
		// We must use another variable to use as check
		// This is becasue we must verify if already readed the directory before.
		let mut dstructure_check: Vec<String> = Vec::new();
    	
    	// Verification variable
    	// alod = at least one directory
    	let mut alod = true;
    	
    	// While "alod" is true keeps in loop
    	while alod {

    	    // Iterate over each "vec" value.
    	    // As then is overwritted we must use it by reference. 
    		for entry in &vec {
    			// Check if is dir.
    			let metadata = fs::metadata(entry).unwrap();

				if metadata.is_dir() {
					if !dstructure.contains(&entry.display().to_string()) {
						dstructure.push( entry.clone().display().to_string() );
					}					
    			} else {
    				// If is file cast it to string and save it in vector.
    				fstructure.push( entry.display().to_string() );
    			}
    		}

    		for entry_n in &dstructure {
    			if !dstructure_check.contains(&entry_n) {
    				dstructure_check.push( entry_n.to_string() );
    				alod = true;
    				vec = readdir ( entry_n.clone() );
    			} else {
    				alod = false;
    			}
    		}

    	}

		dstructure_complete.dbuff = dstructure.clone();
		dstructure_complete.fbuff = fstructure.clone();
    	
    	dstructure_complete
    	
    }

    /// Count words and letters
    /// Return's position; 0 words, 1 letters
    fn word_count(&self) -> Vec<usize> {
        let buffer: Vec<&str>;
        let (mut words, mut letters) = (0, 0);
        let mut wl: Vec<usize> = Vec::new();

        // By default split method only allows one argument, so to put more than one you need
        // specify it as a slice.
        buffer = self
            .split(&[
                ' ', ',', '\t', ':', '@', '#', '<', '>', '(', ')', '/', '=', '!', '"', '$', '%',
                '&', '?',
            ])
            .collect();

        for wr in buffer {
            // This is because when "split" found something of those slice characters, add an ""
            // per character stripped.
            if wr != "" {
                words += 1;
            }
        }

        for ch in self.chars() {
            if ch != '\n' {
                letters += 1;
            }
        }

        wl.push(words);
        wl.push(letters);
        wl
    }

    /// Translate octal permission input to human redeable.
    fn permission_to_human(&self) -> Vec<&'static str> {
        // Positions; 0 setuid/setguid/stickybit, 1 owner, 2 group, 3 others.
        // // Positions of string; 0-1 setguid/setguid/stickybit, 2 owner, 3 group, 4 others
        let mut vecper: Vec<&str> = Vec::new();
        let buffer: Vec<u32> = self
            .clone()
            .chars()
            .map(|d| d.to_digit(10).unwrap())
            .collect();
        let mut fbuffer: Vec<u32> = Vec::new();
        for word in &buffer {
            fbuffer.push(*word);
        }

        // Setuid / Setguid / Stickybit
        match fbuffer[0] {
            0 => vecper.push(" "),
            1 => vecper.push("sticky bit"),
            2 => vecper.push("seted user id"),
            4 => vecper.push("seted group id"),
            _ => vecper.push("error"),
        }

        // Quick fix, because sometimes the octal output (self) is 5 digit and sometimes is 6
        // digit lenght.
        if fbuffer.len() == 5 {
            // Owner
            match fbuffer[2] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Group
            match fbuffer[3] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Others
            match fbuffer[4] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }
        }

        if fbuffer.len() == 6 {
            // Owner
            match fbuffer[3] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Group
            match fbuffer[4] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Others
            match fbuffer[5] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }
        }

        vecper
    }
}

/// Filename is the file's name to open.
/// Input is the string to search
/// Search for "input" into the file and returns strings.
pub fn file_filter(filename: &String, input: String) -> Vec<String> {
    // Read the file as string and then save to lines buffer.
    let mut buffer1 = String::new();

    // Open file
    let mut file = File::open(filename).unwrap();

    // Read file to string and save in buffer1
    match file.read_to_string(&mut buffer1) {
    	// If provides error
    	Err(_e) => {
    		// Show an error about the specific file and cleans the buffer to not break all process.
    		eprintln!("Error to read file; {filename} do not contains valid UTF-8 data");
    		buffer1 = String::new();
    		1	
    	},
    	Ok(d) => d,
    };

    // Split in lines
    let lbuffer = buffer1.lines();

    // If you return directly from "for" loop will be taken as '()'
    // instead String.
    let mut rstr: Vec<String> = Vec::new();

    for word in lbuffer {
        if word.contains(&input) {
            rstr.push(word.to_string());
        }
    }
    rstr
}

/// Unix.
/// Get processes and information.
pub fn getprocs() -> Vec<String> {
	let mut result: Vec<String> = Vec::new();
    // Read the "/proc" directory.
    let mut entries = fs::read_dir("/proc")
        .unwrap()
        // Takes each part and remap it to "path" method to get full path and then extract only the
        // file name of proc's directory.
        .map(|res| {
            res.map(|e| {
                e.path()
                    .file_name()
                    .unwrap()
                    // Transform each file name to str,
                    .to_str()
                    .unwrap()
                    // Transform each str to string.
                    // Take under consideration that as String is stored in heap, to transform
                    // any output to string you need first transform to str.
                    .to_string()
            })
        })
        // Collect each map's returns into a Vec<String>
        .collect::<Result<Vec<String>, io::Error>>()
        // As collect returns a Result this is a must.
        .unwrap();

    // Sort the strings.
    entries.sort();

    // To variables for; procs lists and buffer.
    let mut plist: Vec<String> = Vec::new();
    let mut buff: Vec<String> = Vec::new();
    //let mut fbuff;

    for i in entries {
        // Clean the double quotes ("") at variable's start and end and then pass to String.
        // Then with the file name cleaned, push it to plist vector variable.
        plist.push(i.trim_start_matches('"').trim_end_matches('"').to_string());
    }

    for j in plist {
        // Take each plist's string, split it in chars, go to the first and check if it is a
        // number.
        // If the first char is a number, is appended to strings and then another strings
        // is concatenated to form the final process' status file.
        if j.chars().next().unwrap().is_numeric() {
            buff.push("/proc/".to_string() + &j.to_string() + &"/status".to_string());
        }
    }

    // Sort the processes vector.
    buff.sort();

    // Call to search a word inside a file.
    for file in buff {
        let vname = file_filter(&file, "Name".to_string());
        let vpid = file_filter(&file, "Pid".to_string());
        let vppid = file_filter(&file, "PPid".to_string());
        let vuid = file_filter(&file, "Uid".to_string());
        // A simple way to extract strings from the vector,
        // I know I can use .iter().....etc but with this is more simple and equals quickly.
        for strings in &vname {
            for pid in &vpid {
                for ppid in &vppid {
                    for owner in &vuid {
                        result.push(format!("{}\t{}\t{}\t{}", strings, pid, ppid, owner));
                    }
                }
            }
        }
    }
    result
}
