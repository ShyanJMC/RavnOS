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
use std::collections::HashMap;
// I/O crate
// Buffer reading crate
use std::io::{self,	Write};

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
	let mut rune_history = match OpenOptions::new().create(true).append(true).open( home.clone().to_owned() + "/.ravnos/rune_history" ) {
		Ok(d) => d,
		Err(e) => {
			eprintln!("Error writting / creating .ravnos/rune_history file. \n History will be located in /tmp {e}");
			OpenOptions::new().create(true).append(true).open("/tmp/.ravnos/rune_history" ).expect("Fail creating temporary file")
		},
	};

	// Enabled or not history
	let mut enabled_history: bool = true;

	// The fields of ExitStatus are private, because of that
	// I can not use Rune itself to generate the struct exiting directly.
	let mut coreturn: ExitStatus = process::Command::new( "/usr/bin/sleep" ).arg( "0" ).spawn().expect("").wait().expect("Error stdout command");

	//
	{
		let args = std::env::args();
		for i in args {
			if i == "--help".to_string() || i == "-h".to_string() {
				println!("Rune RavnOS Shell; \n List and/or help builtins; _list");
				process::exit(0);
			}
		}
	}
	////////////////////////////////

	let mut vhistory_map = HashMap::new();
	let mut vhistory_position: usize = 0;

	loop {
		// String vector for history
		let mut vhistory: Vec<String> = match io_mods::get_history(){
			Ok(d) => d,
			Err(_e) => Vec::new(),
		};

		// Map each entry to a position
		let mut temp_buff: usize = 0;
		for i in &vhistory {
				vhistory_map.insert(temp_buff, i.clone());
				temp_buff += 1;
		}

		// Store the position
		if vhistory_position == 0 {
			vhistory_position = vhistory_map.len();
		}

		// Saves the input
		// in each loop is shadowed
		let command: String;

		// Prompt
		let pwd = match std::env::current_dir() {
			Ok(d) => d.display().to_string(),
			Err(e) => e.to_string(),
		};
		print!("[{pwd}]\n> ");
		// Clean the stdout buffer to print the above line before takes the input
		// if not will print first the stdin and then the prompt
		match std::io::stdout().flush() {
			Ok(_d) => (),
			Err(e) => {
				eprintln!("Error cleaning stdout buffer; \n {e}");
				continue;
			}
		}

		command = match io_mods::read_input() {
			Ok(d) => {

				if d == "up_arrow" {
					println!("{}", vhistory_map.get( &(vhistory_position.clone() -1) ).unwrap() );
					vhistory_position -= 1;
					continue;
				} else if d == "down_arrow" {
					println!("{}", vhistory_map.get(&vhistory_position).unwrap() );
					vhistory_position += 1;
					continue;
				} else if d == "right_arrow" {
					println!("\x1B[D");
					continue;
				} else if d == "left_arrow" {
					println!("\x1B[C");
					continue;
				} else {
					d
				}
			},
			Err(e) => {
				eprintln!("{e}");
				continue;
			},
		};

		// Shadowing
		// I directly do here the trim to avoid the same operation in the rest of code so many times
		let mut command = command.trim();
		let mut second_part: &str = "";
		let mut stdout_redirect: Vec<&str> = Vec::new();
		let mut b_stdout_redirect: bool = false;
		let mut stderr_redirect: Vec<&str> = Vec::new();
		let mut b_stderr_redirect: bool = false;
		let mut stdout_file_redirect: Vec<&str> = Vec::new();
		let mut b_stdout_file_redirect: bool = false;

		if command.contains("|") {
			stdout_redirect = command.split("|").map(|e| e.trim()).collect();
			command = stdout_redirect[0];
			second_part = stdout_redirect[1];
			b_stdout_redirect = true;
			drop(stdout_redirect);
		} else if command.contains("2>") {
			stderr_redirect = command.split("2>").map(|e| e.trim()).collect();
			command = stderr_redirect[0];
			second_part = stderr_redirect[1];
			b_stderr_redirect = true;
			drop(stderr_redirect);
		} else if command.contains(">") {
			stdout_file_redirect = command.split(">").map(|e| e.trim()).collect();
			command = stdout_file_redirect[0];
			second_part = stdout_file_redirect[1];
			b_stdout_file_redirect = true;
			drop(stdout_file_redirect);
		}

		let buffer: String;
		// Replace "~" with the home
		if command.contains('~') {
			// Shadowing
			match libstream::search_replace_string(&command.to_string(), &"~".to_string(), &home.to_string() ) {
				Ok(d) => buffer = d,
				Err(_e) => {
					eprintln!("Failing setting ~ to home's user");
					continue;
				}
			};

			command = buffer.as_str();
		}

		if enabled_history {

			if !command.is_empty() {
				let hist_command = {
					let unix_date = match SystemTime::now().duration_since(UNIX_EPOCH){
						Ok(d) => d,
						Err(e) => {
							eprintln!("Error getting duration since UNIX_EPOCH; \n {e}");
							continue;
						},
					}.as_secs();

					// Shadowing
					let unix_date: i64 = unix_date as i64;
					let hist_date = unix_date.epoch_to_human();
					format!("[ {hist_date} ] : {command}")
				};

				// Save the command var after cleaning spaces and tabulations at beggining and
				// end of string.

				match rune_history.write_all(hist_command.as_bytes()) {
					Ok(_d) => (),
					Err(_e) => {
						eprintln!("Error saving command to history file");
						continue;
					},
				}

				match rune_history.write_all(b"\n"){
					Ok(_d) => (),
					Err(_e) => {
						eprintln!("Error saving command to history file");
						continue;
					},
				}

				let mut temp_buff: usize = 0;
					for i in &vhistory {
						vhistory_map.insert(temp_buff, i.clone());
						temp_buff += 1;
					}
				}

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


				if b_stdout_file_redirect {
					// Spawn execute it
					let nproc = match proc.spawn(){
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error executing command. Check if binary/command exists, you can try executing with absolute path.");
							continue;
						}

					};
					// Drop function cleans from memory the variable and data
					//drop(proc);

					// Wait_with_output method waits to command finishes and collect output, return Output struct
					let output = match nproc.wait_with_output() {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error waiting for command and taking process' stdin stdout stderr");
							continue;
						}
					};

					let mut ffile = match OpenOptions::new().create(true).append(true).open(&second_part) {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error creating/opening file");
							continue;
						},
					};

					match ffile.write_all(&output.stdout) {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error writing command's stdout to file");
							continue;
						}
					}
				}
				if b_stderr_redirect {
					// Spawn execute it
					let nproc = match proc.spawn(){
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error executing command. Check if binary/command exists, you can try executing with absolute path.");
							continue;
						}

					};
					// Drop function cleans from memory the variable and data
					//drop(proc);

					// Wait_with_output method waits to command finishes and collect output, return Output struct
					let output = match nproc.wait_with_output() {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error waiting for command and taking process' stdin stdout stderr");
							continue;
						}
					};

					let mut efile = match OpenOptions::new().create(true).append(true).open(&second_part) {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error creating/opening file");
							continue;
						},
					};

					match efile.write_all(&output.stderr) {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error writing command's stdout to file");
							continue;
						}
					}
				}
				if b_stdout_redirect {

					// Take the command, pass to "trim" for clean tabs, spaces and others, split it by spaces and then convert each to string
					// collecting to strings.
					let buff2 = second_part.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
					// Take the first, which is the binary to execute
					let binn2 = &buff2[0].clone();
					let binn2 = binn2.trim();
					let mut arguments2: Vec<String> = Vec::new();

					// TIterate from zero to the size of buff var.
					for i in 0..buff2.len() {
						// Avoid the first, remember, the binary
						if i == 0{
							continue;
						} else {
							arguments2.push(buff2[i].clone());
						}
					}

					// Drop function cleans from memory the variable and data
					drop(buff2);

					// Execute the "binn" binary with the arguments collected in "arguments"
					// "Spawn" execute the command with the arguments. The child process is not lineal to this, will run in
					// 		another core
					let mut proc2 = process::Command::new( binn2.trim() );
					// Arg method inserts the arguments in the Command struct, because of that we can iterate adding it
					for j in arguments2 {
						// Stdout configure process' stdout
						// Stdio::piped connect parent and child processes
						// Stderr configure process' stderr
						// Stdio::piped connect parent and child processes
						proc2.arg(j).stdout(Stdio::piped()).stderr(Stdio::piped());
					}

					// Spawn execute it
					let nproc11 = match proc.spawn(){
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error executing command. Check if binary/command exists, you can try executing with absolute path.");
							continue;
						}

					};

					// output1 takes the normal stdout
					let output1 = nproc11.stdout.unwrap();
					// the output1 stdout is now the stdin for proc2
					proc2.stdin(Stdio::from(output1));

					// Spawn execute it
					let nproc2 = match proc2.spawn(){
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error executing command. Check if binary/command exists, you can try executing with absolute path.");
							continue;
						}

					};
					// Drop function cleans from memory the variable and data
					drop(proc2);


					// Wait_with_output method waits to command finishes and collect output, return Output struct
					let output2 = match nproc2.wait_with_output() {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error waiting for command and taking process' stdin stdout stderr");
							continue;
						}
					};

					match io::stdout().write_all(&output2.stdout) {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error writing command's stdout to system's stdout, the output can not be printed");
							continue;
						}
					}

					coreturn = output2.status;

				}
				if !b_stdout_redirect && !b_stderr_redirect && !b_stdout_file_redirect {
					// Spawn execute it
					let nproc = match proc.spawn(){
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error executing command. Check if binary/command exists, you can try executing with absolute path.");
							continue;
						}

					};
					// Drop function cleans from memory the variable and data
					//drop(proc);

					// Wait_with_output method waits to command finishes and collect output, return Output struct
					let output = match nproc.wait_with_output() {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error waiting for command and taking process' stdin stdout stderr");
							continue;
						}
					};

					match io::stdout().write_all(&output.stdout) {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error writing command's stdout to system's stdout, the output can not be printed");
							continue;
						}
					}

					match io::stderr().write_all(&output.stderr) {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error writing command's stderr to system's stderr, the stderr can not be printed");
							continue;
						}
					}

				}

				if !b_stdout_redirect {
					// Spawn execute it
					let nproc = match proc.spawn(){
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error executing command. Check if binary/command exists, you can try executing with absolute path.");
							continue;
						}

					};
					// Drop function cleans from memory the variable and data
					//drop(proc);

					// Wait_with_output method waits to command finishes and collect output, return Output struct
					let output = match nproc.wait_with_output() {
						Ok(d) => d,
						Err(_e) => {
							eprintln!("Error waiting for command and taking process' stdin stdout stderr");
							continue;
						}
					};
					coreturn = output.status;
				} else {
				}


			} else {
				let proc = match process::Command::new( command.clone() )
											// Stdout configure process' stdout
											// Stdio::piped connect parent and child processes
											.stdout(Stdio::piped())
											// Stderr configure process' stderr
											// Stdio::piped connect parent and child processes
											.stderr(Stdio::piped())
											// Spawn execute it
											.spawn()
											{
												Ok(d) => d,
												Err(_e) => {
													eprintln!("Error executing command. Check if binary/command exists, you can try executing with absolute path.");
													continue;
												}

											};

				// Wait_with_output method waits to command finishes and collect output, return Output struct
				let output = match proc.wait_with_output() {
					Ok(d) => d,
					Err(_e) => {
						eprintln!("Error waiting for command and taking process' stdin stdout stderr");
						continue;
					}
				};

				match io::stdout().write_all(&output.stdout) {
					Ok(d) => d,
					Err(_e) => {
						eprintln!("Error writing command's stdout to system's stdout, the output can not be printed");
						continue;
					}
				}

				match io::stderr().write_all(&output.stderr) {
					Ok(d) => d,
					Err(_e) => {
						eprintln!("Error writing command's stderr to system's stderr, the stderr can not be printed");
						continue;
					}
				}

				coreturn = output.status;


			}

		}

		// we clean the memory dropping the "command" variable
		drop(command);
	} // end loop

}
