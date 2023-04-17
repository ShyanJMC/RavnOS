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

// Process crate
use std::process;
use std::process::ExitStatus;
use std::process::Stdio;
// I/O crate
// Buffer reading crate
use std::io::{self, BufRead, Write, Read};
use std::fs::OpenOptions;

// Import the files inside scope
mod builtins;
mod io_mods;

// For epoch_to_human()
use libstream::Epoch;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn main(){

	// Why do this instead directly store in "home"?
	// Because how memory works; instead use String to store in heap, &str store in stack with a pointer (because of that; &)
	// and if you do not store in a previous variable can unowner that memory when the variable is passed as argument to some
	// function.
	let binding = io_mods::get_user_home();
	let home: &str = binding.as_str().clone();
	let mut rune_history = OpenOptions::new().create(true).append(true).open( home.clone().to_owned() + "/.ravnos/rune_history" ).unwrap();

	// Enabled or not history
	let mut enabled_history: bool = true;

	// The fields of ExitStatus are private, because of that
	// I must use Rune itself to generate the struct exiting directly.
	let mut coreturn: ExitStatus = process::Command::new( "/usr/bin/sleep" ).arg( "0" ).spawn().expect("").wait().expect("Error stdout command");

	//
	{
		let args = std::env::args();
		for i in args {
			if i == "--help".to_string() || i == "-h".to_string() {
				println!("Rune RavnOS Shell; \n List builtins; _list");
				process::exit(0);
			}
		}
	}
	////////////////////////////////

	loop {
		// String vector for history
		let mut vhistory: Vec<String> = Vec::new();
		let mut vhistory_position = vhistory.len();

		// Saves the input
		// in each loop is shadowed
		let mut command = String::new();

		// Prompt
		print!("\n> ");
		// Clean the stdout buffer to print the above line before takes the input
		// if not will print first the stdin and then the prompt
		std::io::stdout().flush().expect("Error cleaning stdout buffer");



		// Lock or not the stdin controlling it
		let mut handle = io::stdin().lock();
		// Read the stdin until newline byte; 0xA byte or EOF (End of file) saving it to "command" variable
		handle.read_line(&mut command).expect("Error reading I/O buffer for stdin");

		// Shadowing
		// I directly do here the trim to avoid the same operation in the rest of code so many times
		let command = command.trim();

		if enabled_history {
			// Temporal memory for history
			// Must be string because we do not know how much large are the commands
			vhistory = match io_mods::get_history(){
				Ok(d) => d,
				Err(_e) => Vec::new(),
			};

			let hist_command = {
				let unix_date = SystemTime::now().duration_since(UNIX_EPOCH).expect("Error getting Unix Epoch Time").as_secs();
				// Shadowing
				let unix_date: i64 = unix_date as i64;
				let hist_date = unix_date.epoch_to_human();
				format!("[ {hist_date} ] : {command}")
			};

			// Save the command var after cleaning spaces and tabulations at beggining and
			// end of string.
			rune_history.write_all(hist_command.as_bytes()).expect("Fail saving history");
			rune_history.write_all(b"\n").expect("Fail saving history");
		}

		// Trim it again and compare with exit string
		if command == "_exit".to_string() {
			process::exit(0);
		} else if command == "_$?".to_string() {
			print!("{}\n", coreturn);
			// Check if start with "_" which means is a builtin
		} else if command == "_history".to_string() && enabled_history {
			let mut num = 0;
			for i in &vhistory {
				println!("{num} {i}");
				num +=1;
			}
		} else if command == "_disable_history" {
			enabled_history = false;
			vhistory.clear();
		} else if command == "_enable_history" {
			enabled_history = true;
		// To avoid that the shell execute a space or a new line
		} else if command == " " || command == "\n" || command == "" {
			// Do not misunderstand, "continue" breaks the actual loop to start again
			continue;
		} else if command == "_home".to_string() {
			print!("{home}");
		} else if command.chars().next().unwrap() == '_' {
				// With this we take the characters avoiding the first; "_"
				let buff = &command[1..].split(' ').collect::<Vec<&str>>().clone();
				let builtin = buff[0].clone();
				// For arguments we take the complete vector and then we remove the first position
				// which is the binary. Then we strip the '\n' at the end of string
				let b_arguments: String = {
					let mut args = buff.clone();
					args.remove(0);
					let temp = args.join(" ");
					let temp = temp.trim_end_matches('\n').to_string();
					temp
				};

				// Drop function cleans from memory the variable and data
				drop(buff);

				match builtins::rbuiltins( builtin, b_arguments ) {
					Ok(d) => println!("{d}"),
					Err(e) => println!("{e}"),
				}
		} else {
			if command.len() > 2 {
				// Take the command, pass to "trim" for clean tabs, spaces and others, split it by spaces and then convert each to string
				// collecting to strings.
				let buff = command.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
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
					proc.arg(j).stdout(Stdio::piped()).stderr(Stdio::piped());
				}
				// Spawn execute it
				let nproc = proc.spawn().expect("Error executing command.");
				// Drop function cleans from memory the variable and data
				drop(proc);

				// Wait_with_output method waits to command finishes and collect output, return Output struct
				let output = nproc.wait_with_output().expect("Error waiting for command and taking process' stdin stdout stderr");

				io::stdout().write_all(&output.stdout).unwrap();
				io::stderr().write_all(&output.stderr).unwrap();

				coreturn = output.status;


			} else {
				let proc = process::Command::new( command.clone() )
											// Stdout configure process' stdout
											// Stdio::piped connect parent and child processes
											.stdout(Stdio::piped())
											// Stderr configure process' stderr
											// Stdio::piped connect parent and child processes
											.stderr(Stdio::piped())
											// Spawn execute it
											.spawn().expect("Error executing command.");

				// Wait_with_output method waits to command finishes and collect output, return Output struct
				let output = proc.wait_with_output().expect("Error waiting for command and taking process' stdin stdout stderr");

				io::stdout().write_all(&output.stdout).unwrap();
				io::stderr().write_all(&output.stderr).unwrap();

				coreturn = output.status;


			}

		}

		// we clean the memory dropping the "command" variable
		drop(command);
	} // end loop

}
