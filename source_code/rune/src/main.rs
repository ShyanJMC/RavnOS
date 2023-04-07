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
use std::io::{self, BufRead, Write};

// Import the files inside scope
mod builtins;

fn main(){
	// Temporal memory for history
	// Must be string because we do not know how much large are the commands
	let mut vhistory: Vec<String> = Vec::new();

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

		// Prompt
		print!("> ");
		// Clean the stdout buffer to print the above line before takes the input
		// if not will print first the stdin and then the prompt
		std::io::stdout().flush().expect("Error cleaning stdout buffer");

		// Saves the input
		// in each loop is shadowed
		let mut command = String::new();

		// Lock or not the stdin controlling it
		let mut handle = io::stdin().lock();
		// Read the stdin until newline byte; 0xA byte or EOF (End of file) saving it to "command" variable
		handle.read_line(&mut command).expect("Error reading I/O buffer for stdin");

		// Save the command in Vec<String> vector after cleaning spaces and tabulations at beggining and
		// end of string. To avoid issues with ownership we use the to_string method to duplicate the memory (the same as clone).
		vhistory.push( command.trim().to_string() );

		// Trim it again and compare with exit string
		if command.trim() == "exit".to_string() {
			process::exit(0);

		} else if command.trim() == "$?".to_string() {
			print!("{}\n", coreturn);
			// Check if start with "_" which means is a builtin
		} else if command.chars().next().unwrap().to_string() == "_".to_string() {
				// With this we take the characters avoiding the first; "_"
				let builtin = &command[1..];
				match builtins::rbuiltins( builtin ) {
					Ok(d) => println!("{d}"),
					Err(e) => println!("{e}"),
				}
		} else {
			if command.trim().len() > 2 {
				// Take the command, pass to "trim" for clean tabs, spaces and others, split it by spaces and then convert each to string
				// collecting to strings.
				let buff = command.trim().split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
				// Take the first, which is the binary to execute
				let binn = &buff[0].trim();
				let mut arguments: String = String::new();

				// TIterate from zero to the size of buff var.
				for i in 0..buff.len() {
					// Avoid the first, remember, the binary
					if i == 0{
						continue;
					} else {
						//  Appens the strings
						if arguments.len() != 0 {
							arguments = arguments + &" ".to_string() + &buff[i];
						} else {
							arguments = buff[i].to_string();
						}
					}
				}

				// Execute the "binn" binary with the arguments collected in "arguments"
				// "Spawn" execute the command with the arguments. The child process is not lineal to this, will run in
				// 		another core
				// let mut proc = process::Command::new( binn ).arg( arguments.clone() ).spawn().expect("Error stdout command.");
				let mut proc = process::Command::new( binn.trim() ).arg( arguments.clone() )
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


			// To avoid that the shell execute a space or a new line
			} else if command == " " || command == "\n" {
				continue;

			} else {
				let mut proc = process::Command::new( command.clone().trim() )
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
	}

}
