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
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022-2023


// Process crate
use std::process::{self,ChildStderr,ChildStdout,ExitStatus,Stdio};
// HashMap
use std::collections::HashMap;
// IO lib
use std::io::{self,prelude::*};
// Thread and Sync libs
use std::sync::{Arc, Mutex};
use std::thread::{self,JoinHandle};
use std::sync::mpsc::channel;

// Colors
use libstream::Colors;

pub struct SService {
    id: i64,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
	exitstatus: ExitStatus,
}

// Mutex is used to allow only one thread to change the value at the same time
// Arc is used to allow ownership between more that one thread.
/////////////////////////////////////

pub fn thread_command( h_s_service: HashMap<String,i64>, name: String, binary: String) -> JoinHandle<Result<SService, String>>{
		thread::spawn(move || {
			let mut h_s_service = h_s_service.clone();
			let color = Colors::new();
			let s_id: i64;

	        // Take the command, pass to "trim" for clean tabs, spaces and others, split it by spaces and then convert each to string
	        // collecting to strings.
	        let buff = binary.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
	        // Take the first, which is the binary to execute
	        let binn = &buff[0].clone();
	        let binn = binn.trim();
	        let mut arguments: Vec<String> = Vec::new();

	        // TIterate from zero to the size of buff var.
	        for i in 0..buff.len() {
	            // Avoid the first, remember, the binary
	            if i == 0{
	                continue;
	            } else {
	                arguments.push(buff[i].clone());
	            }
	        }

	        // Drop function cleans from memory the variable and data
	        drop(buff);

	        // Execute the "binn" binary with the arguments collected in "arguments"
	       // "Spawn" execute the command with the arguments. The child process is not lineal to this, will run in
	        // 		another core
	        let mut proc = process::Command::new( binn.trim() );
	        // Arg method inserts the arguments in the Command struct, because of that we can iterate adding it
	        for j in arguments {
	            // Stdout configure process' stdout
	            // Stdio::piped connect parent and child processes
	            // Stderr configure process' stderr
	            // Stdio::piped connect parent and child processes
	            proc.arg(j).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
	        }


	        println!("{}[INFO]\tStarting; {name}{}", color.cyan, color.reset);
	        // Spawn execute it
	        let nproc = match proc.spawn(){
	            Ok(d) => {
	                s_id = d.id().into();
	                h_s_service.insert( name.clone(), s_id );
	                println!("{}[OK]\tStarted {name}, id; {s_id}{}", color.green, color.reset);
	                d
	            },
	            Err(e) => {
	                eprintln!("{}[ERR]\t\tError executing {name}. Check if binary/command exists, you can try executing with absolute path.{}", color.red, color.reset);
	                eprintln!("{}[ERR][INFO]\t{name} status; {e}{}", color.red, color.reset);
					return Err("".to_string());
	            },

	        };
	        // Drop function cleans from memory the variable and data
	        drop(proc);


	        // Wait_with_output method waits to command finishes and collect output, return Output struct
	        let output = match nproc.wait_with_output() {
	            Ok(d) => d,
	            Err(_e) => {
	                eprintln!("{}[ERR]\t\t{name} Error waiting for command and taking process' stdin stdout stderr{}", color.red, color.reset);
					return Err("".to_string());
	            }
	        };

			// Return this as Ok(SService)
			let instance = SService {
			    id: s_id,
			    stdout: output.stdout,
			    stderr: output.stderr,
				exitstatus: output.status,
			};
	        Ok(instance)

		})
}


pub fn single_command( h_s_service:&mut HashMap<String,i64>, name: String, binary: String) -> Result<SService,String>{

		let color = Colors::new();
		let s_id: i64;

        // Take the command, pass to "trim" for clean tabs, spaces and others, split it by spaces and then convert each to string
        // collecting to strings.
        let buff = binary.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
        // Take the first, which is the binary to execute
        let binn = &buff[0].clone();
        let binn = binn.trim();
        let mut arguments: Vec<String> = Vec::new();

        // TIterate from zero to the size of buff var.
        for i in 0..buff.len() {
            // Avoid the first, remember, the binary
            if i == 0{
                continue;
            } else {
                arguments.push(buff[i].clone());
            }
        }

        // Drop function cleans from memory the variable and data
        drop(buff);

        // Execute the "binn" binary with the arguments collected in "arguments"
       // "Spawn" execute the command with the arguments. The child process is not lineal to this, will run in
        // 		another core
        let mut proc = process::Command::new( binn.trim() );
        // Arg method inserts the arguments in the Command struct, because of that we can iterate adding it
        for j in arguments {
            // Stdout configure process' stdout
            // Stdio::piped connect parent and child processes
            // Stderr configure process' stderr
            // Stdio::piped connect parent and child processes
            proc.arg(j).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        }


        println!("{}[INFO]\tStarting; {name}{}", color.cyan, color.reset);
        // Spawn execute it
        let nproc = match proc.spawn(){
            Ok(d) => {
                s_id = d.id().into();
                h_s_service.insert( name.clone(), s_id );
                println!("{}[INFO]\tStarted, id; {s_id}{}", color.green, color.reset);
                d
            },
            Err(e) => {
                eprintln!("{}[ERR]\t\tError executing {name}. Check if binary/command exists, you can try executing with absolute path.{}", color.red, color.reset);
                eprintln!("{}[ERR][INFO]\t{name} status; {e}{}", color.red, color.reset);
				return Err("".to_string());
            },

        };
        // Drop function cleans from memory the variable and data
        drop(proc);


        // Wait_with_output method waits to command finishes and collect output, return Output struct
        let output = match nproc.wait_with_output() {
            Ok(d) => d,
            Err(_e) => {
                eprintln!("{}[ERR]\t\t{name} Error waiting for command and taking process' stdin stdout stderr{}", color.red, color.reset);
				return Err("".to_string());
            }
        };

		// Return this as Ok(SService)
		let instance = SService {
		    id: s_id,
		    stdout: output.stdout,
		    stderr: output.stderr,
			exitstatus: output.status,
		};
        Ok(instance)

}
