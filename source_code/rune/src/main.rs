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

use std::fs::{File,OpenOptions};

use std::path::Path;

// Import the files inside scope
mod builtins;
mod io_mods;

// For epoch_to_human()
use libstream::Epoch;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub struct SService {
    stdout: String,
    stderr: String,
}

fn main(){
	let mut rune_history = match OpenOptions::new().create(true).append(true).open( io_mods::get_user_home() + "/.ravnos/rune_history" ) {
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
		let mut string_stdout: String;
		let mut string_stderr: String;
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
		let mut second_part = "";
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
			match libstream::search_replace_string(&command.to_string(), &"~".to_string(), &io_mods::get_user_home() ) {
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
		} else if command == "_history".to_string() && enabled_history {
			let mut num = 0;
			for i in &vhistory {
				println!("{}", format!("{num} {i}"));
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
		} else {
			if !second_part.is_empty() {

				if b_stdout_file_redirect {
					let mut command_stdin = command.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
					command_stdin.remove(0);
					let stdreturn = fcommand(command, command_stdin, Vec::new());
					string_stdout = stdreturn.stdout;
					string_stderr = stdreturn.stderr;

					OpenOptions::new().create(true).append(true).open(second_part).unwrap();
					let _ = std::fs::write(Path::new(second_part), string_stdout);
					eprintln!("{string_stderr}");

				} else if b_stderr_redirect {
					let mut command_stdin = command.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
					command_stdin.remove(0);
					let stdreturn = fcommand(command, command_stdin, Vec::new());
					string_stdout = stdreturn.stdout;
					string_stderr = stdreturn.stderr;

					OpenOptions::new().create(true).append(true).open(second_part).unwrap();
					let _ = std::fs::write(Path::new(second_part), string_stderr);
					println!("{string_stdout}");
				}

				if b_stdout_redirect {
					let mut command_stdin = command.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
					command_stdin.remove(0);
					let stdreturn = fcommand(command, command_stdin, Vec::new());
					string_stdout = stdreturn.stdout;
					string_stderr = stdreturn.stderr;

					let mut second_part_args = second_part.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
                    let borrow = second_part_args[0].clone();
					let second_part_bin = (borrow.split(' ').collect::<Vec<&str>>())[0];
					second_part_args.remove(0);

                    let stdin_data = {
                        let mut vec = Vec::new();
                        vec.push(string_stdout.as_str());
                        vec
                    };

					let stdreturn2 = fcommand(&second_part_bin, second_part_args, stdin_data);
					let string_stdout2 = stdreturn2.stdout;
					let string_stderr2 = stdreturn2.stderr;

                    if !string_stderr.is_empty() && string_stdout.is_empty(){
                        eprintln!("stderr {{\n{string_stderr2}}}\n");
                    } else if !string_stderr.is_empty() && !string_stdout.is_empty(){
                        println!("stdout {{\n{string_stdout2}}}\n");
                        eprintln!("stderr {{\n{string_stderr2}}}\n");
                    } else if string_stderr.is_empty() {
                        println!("{string_stdout2}");
                    }

				}
			} else {
				let mut arguments = command.split(' ').map(|e| e.to_string()).collect::<Vec<String>>();
				arguments.remove(0);
				let stdreturn = fcommand(command, arguments, Vec::new());
				string_stdout = stdreturn.stdout;
				string_stderr = stdreturn.stderr;

                if !string_stderr.is_empty() && string_stdout.is_empty(){
                    eprintln!("stderr {{\n{string_stderr}}}\n");
                } else if !string_stderr.is_empty() && !string_stdout.is_empty(){
                    println!("stdout {{\n{string_stdout}}}\n");
                    eprintln!("stderr {{\n{string_stderr}}}\n");
                } else if string_stderr.is_empty() {
                    println!("{string_stdout}");
                }
			}

		}
	} // end loop

}

pub fn fcommand( input: &str, arguments: Vec<String>, stdin_data: Vec<&str>) -> SService {
    let mut init1 = SService {
      stdout: String::new(),
      stderr: String::new(),
    };

    // Take the first, which is the binary to execute
    let binn = {
        if input.chars().next().unwrap() == '_' {
                // With this we take the characters avoiding the first; "_"
                let buff: Vec<&str> = input[1..].split(' ').collect::<Vec<&str>>();
                let builtin = buff[0];
                // For arguments we take the complete vector and then we remove the first position
                // which is the binary. Then we strip the '\n' at the end of string
                let b_arguments: String = {
                    let mut args = buff.clone();
                    args.remove(0);
                    let temp = args.join(" ");
                    let temp = temp.trim_end_matches('\n').to_string();
                    temp
                };

                match builtins::rbuiltins( builtin, b_arguments ) {
                    Ok(d) => {
						init1.stdout = d.clone().trim().to_string();
						return init1;
					}
                    Err(e) => {
						init1.stderr = e.trim().to_string();
						return init1;
					},
                }
        } else if !input.contains("/bin/"){
				match builtins::rbuiltins( "which", (input.split(' ').map(|e| e.to_string()).collect::<Vec<String>>())[0].clone() ) {
					Ok(d) => d,
					Err(_e) => {
						eprintln!("Binary {:?} not found in PATH", input);
						return init1;
					},
				}
		} else {
				(input.split(' ').map(|e| e.to_string()).collect::<Vec<String>>())[0].clone().to_string()
		}
    };

    // Execute the "binn" binary with the arguments collected in "arguments"
    let mut proc = process::Command::new( binn );
    // Arg method inserts the arguments in the Command struct, because of that we can iterate adding it
    for j in &arguments {
        // Stdout configure process' stdout
        // Stdio::piped connect parent and child processes
        // Stderr configure process' stderr
        // Stdio::piped connect parent and child processes
        proc.arg(j).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    }

    if stdin_data.is_empty(){
        // Spawn execute it
        let nproc = proc.spawn().unwrap();
        // Wait_with_output method waits to command finishes and collect output, return Output struct
        let output = nproc.wait_with_output().unwrap();
        init1.stdout = std::str::from_utf8(&(output.stdout).clone()).unwrap().to_string();
        init1.stderr = std::str::from_utf8(&(output.stderr).clone()).unwrap().to_string();
    } else {
        // Spawn execute it
        let mut nproc = proc.spawn().expect("Error al ejecutar el proceso");
        if let Some(ref mut nproc_stdin) = nproc.stdin {
            for i in stdin_data {
                nproc_stdin.write_all(i.as_bytes()).expect("Error al escribir en stdin");
            }
        } else {
            eprintln!("Error taking child's stdin");
            return init1;
        }
        // Wait_with_output method waits to command finishes and collect output, return Output struct
        let output = nproc.wait_with_output().unwrap();
        init1.stdout = std::str::from_utf8(&(output.stdout).clone()).unwrap().to_string();
        init1.stderr = std::str::from_utf8(&(output.stderr).clone()).unwrap().to_string();
    }


    init1
}
