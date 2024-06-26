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
use std::process::Stdio;

// Colections crate
use std::collections::HashMap;
use std::collections::VecDeque;
// I/O crate
// Buffer reading crate
use std::io::Write;

use std::fs::{self,OpenOptions};

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

	let mut vhistory_map = HashMap::new();
	let mut vhistory_position: usize = 0;
	let halias = io_mods::aliases();

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
		let mut command: String;

		// Prompt
		let prompt_pwd = match std::env::current_dir() {
			Ok(d) => d.display().to_string(),
			Err(e) => e.to_string(),
		};
        let prompt_user = {
            let mut varvalue: String = String::new();
            for (key, value) in std::env::vars(){
                if key == "USER" {
                    varvalue = String::from( value);
                    break;
                }
            }
            varvalue
        };
		print!("[{prompt_pwd}]\n{prompt_user} > ");
        
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

		for (k,v) in &halias {
			command = match libstream::search_replace_string(&command, &k, &v) {
				Ok(d) => d,
				Err(_e) => command,
			};
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
			command = buffer;
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
            // commargs = command and arguments
            // Is a vector of strings because each position is one command
            let mut commargs: Vec<String> = Vec::new();
			// I use here a vector to store commands and redirection
			// is stored in the same position that was introduced
			// you need to be tidy/prolix and use spaces when correspond
		    let command_map: Vec<&str> = command.split(' ').collect();

            // match_indices returns every position in a vector (k,v)
            let mut ampersand_redirect_position = {
                let mut buffer = VecDeque::new();
                for (k,_v) in command.match_indices("&&") {
                    buffer.push_back(k);
                }
                buffer
            };
            let mut stdout_redirect_position;
            let mut stderr_redirect_position;
            let mut pipeline_redirect_position;

            // Split the input by ampersands (&&) and semicolons (;)
            // saving them into "commargs"
            if command_map.contains(&"&&") || command_map.contains(&";"){
		    	let mut buffer1 = String::new();
                for i in &command_map {
                    if i != &";" && i != &"&&" {
                        if !buffer1.is_empty(){
                            buffer1 = buffer1 + " " + &i;
                        } else {
                            buffer1 = i.to_string();
                        }
                    } else {
                        commargs.push(buffer1.clone());
                        buffer1.clear();
                    }
                }
                // This is because when ends command_map by default do not add
                // it when comes to end of string
                commargs.push(buffer1.clone());
                buffer1.clear();
            }

			// I use this to store if the last command terminated successfully
			let mut last_return: bool = true;

		    if !commargs.is_empty(){
		    	for vcommand in commargs {
                    let mut buffering: Vec<&str>  = vcommand.split(' ').collect();
                    // Iterate over vector and check if there are some empty
                    // value, as "position" returns Some(d) is needed for check
                    // use "if let".
                    if let Some(d) = buffering.iter().position(|e| e.is_empty()) {
                        buffering.remove(d);
                    }

                    // Store the position of next stdout, stderr or pipeline
                    // redirection. If there are not, store the last position
                    stderr_redirect_position = {
                        match buffering.iter().position(|e| e == &"2>") {
                            Some(d) => d,
                            None => 0,
                        }
                    };
                    stdout_redirect_position = {
                        match buffering.iter().position(|e| e == &">") {
                            Some(d) => d,
                            None => 0,
                        }
                    };
                    pipeline_redirect_position = {
                        match buffering.iter().position(|e| e == &"|") {
                            Some(d) => d,
                            None => 0,
                        }
                    };

                    let fposition = {
                        let mut buffer = Vec::new();
                        if stdout_redirect_position != 0 {
                            buffer.push(stdout_redirect_position);
                        }
                        if stderr_redirect_position != 0 {
                            buffer.push(stderr_redirect_position);
                        }
                        if pipeline_redirect_position != 0 {
                            buffer.push(pipeline_redirect_position);
                        }
                        buffer.sort();
                        if !buffer.is_empty() {
                            buffer[0]
                        } else {
                            buffering.len()
                        }

                    };

                    // Store the binary and arguments in vector
                    // First is the binary and the second is the arguments
                    let cargs = {
                        let mut bvec: Vec<String> = Vec::new();
                        bvec.push( buffering[0].to_string() );

                        let mut buffer2 = String::new();
                        for j in 1..fposition{
                            if !buffer2.is_empty(){
                                buffer2 = buffer2 + " " + &buffering[j];
                            } else {
                                buffer2 = buffering[j].to_string();
                            }
                        }
                        bvec.push( buffer2 );
                        bvec
                    };

                    let stdout_file = if stdout_redirect_position != 0 {
                        buffering[stdout_redirect_position+1]
                    } else {
                        ""
                    };


                    let stderr_file = if stderr_redirect_position != 0 {
                        buffering[stderr_redirect_position+1]
                    } else {
                        ""
                    };

                    // Store the second command that will use pipeline
                    let second_command = if pipeline_redirect_position != 0 {
                        let mut buffer = String::new();
                        for i in (pipeline_redirect_position+1)..buffering.len() {
                            if buffering[i] != "2>" && buffering[i] != ";" {
                                if !buffer.is_empty() {
                                    buffer = buffer + " " + &buffering[i];
                                } else {
                                    buffer = buffering[i].to_string();
                                }
                            }
                        }
                        buffer
                    } else {
                        "".to_string()
                    };

                    // Execute depending all before
                    // ampersand_redirect_position = vector
                    // last_return = boolean
                    // cargs = string
                    // stdout_file = string
                    // stderr_file = string
                    // second_command = string
                    //
                    // Check if the position cargs is the same that ampersand
                    // Takes the binary and arguments in the same string
                    let tempbuff = {
                        let mut buffer: String = String::new();
                        if !buffer.is_empty(){
                            buffer = buffer + &cargs[1].to_string();
                        } else {
                            buffer = cargs[0].clone();
                        }
                        buffer
                    };
                    // Takes the position of binary and arguments to execute
                    let command_position = {
                        let mut keysv = Vec::new();
                        for (k,_v) in command.match_indices(&tempbuff) {
                            keysv.push(k);
                        }
                        keysv
                    };

                    // First command
                    if command_position[0] == 0{
                        if second_command.is_empty(){
                            let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                            let empty: Vec<&str> = Vec::new();
                            let freturn = fcommand(&cargs[0], arguments, empty);
                            if freturn.stderr.is_empty(){
                                last_return = true;
                            } else if !stderr_file.is_empty(){
                                last_return = false;
                                let _ = fs::write(stderr_file, freturn.stderr );
                            } else {
                                last_return = false;
                                eprintln!("{}", freturn.stderr);
                            }
                            if stdout_file.is_empty(){
                                println!("{}", freturn.stdout);
                            } else {
                                let _ = fs::write(stdout_file, freturn.stdout );
                            }

                        } else {
                            // Pipeline redirect
                            let second_bin = second_command.split(' ').collect::<Vec<&str>>()[0];
                            let second_argument: Vec<String> = {
                                let mut buffer = String::new();
                                for i in second_command.split(' ').collect::<Vec<&str>>() {
                                    if i != second_bin {
                                        if i == ">" || i == "2>" {
                                            break;
                                        }
                                        if !buffer.is_empty(){
                                            buffer = buffer + " " + &i.to_string();
                                        } else {
                                            buffer = i.to_string();
                                        }
                                    }
                                }
                                buffer.split(' ').map(|e| e.to_string()).collect()
                            };

                            // First part
                            let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                            let empty: Vec<&str> = Vec::new();
                            let freturn = fcommand(&cargs[0], arguments, empty);
                            let mut second_stdin: Vec<&str> = Vec::new();
                            if !freturn.stderr.is_empty(){
                                second_stdin.push(freturn.stderr.as_str());
                            }
                            if !freturn.stdout.is_empty(){
                                for i in freturn.stdout.lines() {
                                    second_stdin.push(i);
                                }
                            }

                            let freturn2 = fcommand(&second_bin, second_argument, second_stdin);
                            if freturn2.stderr.is_empty(){
                                last_return = true;
                            } else if !stderr_file.is_empty(){
                                last_return = false;
                                let _ = fs::write(stderr_file, freturn2.stderr );
                            } else {
                                last_return = false;
                                eprintln!("{}", freturn2.stderr);
                            }
                            if stdout_file.is_empty(){
                                println!("{}", freturn2.stdout);
                            } else {
                                let _ = fs::write(stdout_file, freturn2.stdout );
                            }
                        }
                    // Second or higher command
                    } else {
                        if !ampersand_redirect_position.is_empty() && (command_position[0]-3) == ampersand_redirect_position[0] {
                        	let _ = ampersand_redirect_position.pop_front();
                            if last_return == true {
                                if second_command.is_empty(){
                                    let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                                    let empty: Vec<&str> = Vec::new();
                                    let freturn = fcommand(&cargs[0], arguments, empty);
                                    if freturn.stderr.is_empty(){
                                        last_return = true;
                                    } else if !stderr_file.is_empty(){
                                        let _ = fs::write(stderr_file, freturn.stderr );
                                    } else {
                                        last_return = false;
                                        eprintln!("{}", freturn.stderr);
                                    }

                                    if stdout_file.is_empty(){
                                        println!("{}", freturn.stdout);
                                    } else {
                                        let _ = fs::write(stdout_file, freturn.stdout );
                                    }
                                } else {
                                    // Pipeline redirect
                                    let second_bin = second_command.split(' ').collect::<Vec<&str>>()[0];
                                    let second_argument: Vec<String> = {
                                        let mut buffer = String::new();
                                        for i in second_command.split(' ').collect::<Vec<&str>>() {
                                            if i != second_bin {
                                                if i == ">" || i == "2>" {
                                                    break;
                                                }
                                                if !buffer.is_empty(){
                                                    buffer = buffer + " " + &i.to_string();
                                                } else {
                                                    buffer = i.to_string();
                                                }
                                            }
                                        }
                                        buffer.split(' ').map(|e| e.to_string()).collect()
                                    };

                                    // First part
                                    let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                                    let empty: Vec<&str> = Vec::new();
                                    let freturn = fcommand(&cargs[0], arguments, empty);
                                    let mut second_stdin: Vec<&str> = Vec::new();
                                    if !freturn.stderr.is_empty(){
                                        second_stdin.push(freturn.stderr.as_str());
                                    }
                                    if !freturn.stdout.is_empty(){
                                        for i in freturn.stdout.lines() {
                                            second_stdin.push(i);
                                        }
                                    }

                                    let freturn2 = fcommand(&second_bin, second_argument, second_stdin);
                                    if freturn2.stderr.is_empty(){
                                        last_return = true;
                                    } else if !stderr_file.is_empty(){
                                        last_return = false;
                                        let _ = fs::write(stderr_file, freturn2.stderr );
                                    } else {
                                        last_return = false;
                                        eprintln!("{}", freturn2.stderr);
                                    }
                                    if stdout_file.is_empty(){
                                        println!("{}", freturn2.stdout);
                                    } else {
                                        let _ = fs::write(stdout_file, freturn2.stdout );
                                    }
                                }
                            } else {
                                eprintln!("ERR: {} {} not executed",cargs[0],cargs[1]);
                            }
                        } else {
                            if second_command.is_empty(){
                              let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                              let empty: Vec<&str> = Vec::new();
                              let freturn = fcommand(&cargs[0], arguments, empty);
                              if freturn.stderr.is_empty(){
                                  last_return = true;
                              } else if !stderr_file.is_empty(){
                                  let _ = fs::write(stderr_file, freturn.stderr );
                              } else {
                                  last_return = false;
                                  eprintln!("{}", freturn.stderr);
                              }

                              if stdout_file.is_empty(){
                                  println!("{}", freturn.stdout);
                              } else {
                                  let _ = fs::write(stdout_file, freturn.stdout );
                              }

                            } else {
                                // Pipeline redirect
                                // First part

                                let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                                let empty: Vec<&str> = Vec::new();
                                let freturn = fcommand(&cargs[0], arguments, empty);

                                let second_bin = second_command.split(' ').collect::<Vec<&str>>()[0];
                                let second_argument: Vec<String> = {
                                	let mut buffer = String::new();
                                	for i in second_command.split(' ').collect::<Vec<&str>>() {
                                	if i != second_bin {
                                		if i == ">" || i == "2>" {
                                			break;
                                		}

                                		if !buffer.is_empty(){
                                			buffer = buffer + " " + &i.to_string();
                                		} else {
                                			buffer = i.to_string();
                                		}
                                     }
                                    }
                                    buffer.split(' ').map(|e| e.to_string()).collect()
                                };

                                let mut second_stdin: Vec<&str> = Vec::new();
                                if !freturn.stderr.is_empty(){
                                	second_stdin.push(freturn.stderr.as_str());
                                }

                                if !freturn.stdout.is_empty(){
                                	for i in freturn.stdout.lines() {
                                		second_stdin.push(i);
                                    }
                                }

                                let freturn2 = fcommand(&second_bin, second_argument, second_stdin);

                                if !stderr_file.is_empty(){
                                	let _ = fs::write(stderr_file, freturn2.stderr );
                                } else {
                                	last_return = false;
                                    eprintln!("{}", freturn2.stderr);
                                }

                                if stdout_file.is_empty(){
                                    println!("{}", freturn2.stdout);
                                } else {
                                    let _ = fs::write(stdout_file, freturn2.stdout );
                                }
                          }
                        }

                    }
                 }

		    } else {
                // Single command

                let mut buffering: Vec<&str>  = command.split(' ').collect();
                // Iterate over vector and check if there are some empty
                // value, as "position" returns Some(d) is needed for check
                // use "if let".
                if let Some(d) = buffering.iter().position(|e| e.is_empty()) {
                    buffering.remove(d);
                }

                // Store the position of next stdout, stderr or pipeline
                // redirection. If there are not, store the last position
                stderr_redirect_position = {
                    match buffering.iter().position(|e| e == &"2>") {
                        Some(d) => d,
                        None => 0,
                    }
                };
                stdout_redirect_position = {
                    match buffering.iter().position(|e| e == &">") {
                        Some(d) => d,
                        None => 0,
                    }
                };
                pipeline_redirect_position = {
                    match buffering.iter().position(|e| e == &"|") {
                        Some(d) => d,
                        None => 0,
                    }
                };

                let fposition = {
                    let mut buffer = Vec::new();
                    if stdout_redirect_position != 0 {
                        buffer.push(stdout_redirect_position);
                    }
                    if stderr_redirect_position != 0 {
                        buffer.push(stderr_redirect_position);
                    }
                    if pipeline_redirect_position != 0 {
                        buffer.push(pipeline_redirect_position);
                    }
                    buffer.sort();
                    if !buffer.is_empty() {
                        buffer[0]
                    } else {
                        buffering.len()
                    }

                };
                // Store the binary and arguments in vector
                // First is the binary and the second is the arguments
                let cargs = {
                    let mut bvec: Vec<String> = Vec::new();
                    bvec.push( buffering[0].to_string() );

                    let mut buffer2 = String::new();
                    for j in 1..fposition{
                        if !buffer2.is_empty(){
                            buffer2 = buffer2 + " " + &buffering[j];
                        } else {
                            buffer2 = buffering[j].to_string();
                        }
                    }
                    bvec.push( buffer2 );
                    bvec
                };

                let stdout_file = if stdout_redirect_position != 0 {
                    buffering[stdout_redirect_position+1]
                } else {
                    ""
                };


                let stderr_file = if stderr_redirect_position != 0 {
                    buffering[stderr_redirect_position+1]
                } else {
                    ""
                };

                // Store the second command that will use pipeline
                let second_command = if pipeline_redirect_position != 0 {
                    let mut buffer = String::new();
                    for i in (pipeline_redirect_position+1)..buffering.len() {
                        if buffering[i] != "2>" && buffering[i] != ";" {
                            if !buffer.is_empty() {
                                buffer = buffer + " " + &buffering[i];
                            } else {
                                buffer = buffering[i].to_string();
                            }
                        }
                    }
                    buffer
                } else {
                    "".to_string()
                };
                if second_command.is_empty(){
                    let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                    let empty: Vec<&str> = Vec::new();
                    let freturn = fcommand(&cargs[0], arguments, empty);
                    if !stderr_file.is_empty(){
                        let _ = fs::write(stderr_file, freturn.stderr );
                    } else {
                    	eprintln!("{}", freturn.stderr);
                    }
                    if stdout_file.is_empty(){
                        println!("{}", freturn.stdout);
                    } else {
                        let _ = fs::write(stdout_file, freturn.stdout );
                    }

                } else {
                    // Pipeline redirect

                    // First part
                    let arguments: Vec<String> = cargs[1].split(' ').map(|e| e.to_string()).collect();
                    let empty: Vec<&str> = Vec::new();
                    let freturn = fcommand(&cargs[0], arguments, empty);

                    let second_bin = second_command.split(' ').collect::<Vec<&str>>()[0];
                    let second_argument: Vec<String> = {
                        let mut buffer = String::new();
                        for i in second_command.split(' ').collect::<Vec<&str>>() {
                            if i != second_bin {
                                if i == ">" || i == "2>" {
                                    break;
                                }
                                if !buffer.is_empty(){
                                    buffer = buffer + " " + &i.to_string();
                                } else {
                                    buffer = i.to_string();
                                }
                            }
                        }
                        buffer.split(' ').map(|e| e.to_string()).collect()
                    };

                    let mut second_stdin: Vec<&str> = Vec::new();
                    if !freturn.stderr.is_empty(){
                        second_stdin.push(freturn.stderr.as_str());
                    }
                    if !freturn.stdout.is_empty(){
                        for i in freturn.stdout.lines() {
                            second_stdin.push(i);
                        }
                    }

                    let freturn2 = fcommand(&second_bin, second_argument, second_stdin);

                    if !stderr_file.is_empty(){
                        let _ = fs::write(stderr_file, freturn2.stderr );
                    } else {
                        eprintln!("{}", freturn2.stderr);
                    }

                    if stdout_file.is_empty(){
                        println!("{}", freturn2.stdout);
                    } else {
                        let _ = fs::write(stdout_file, freturn2.stdout );
                    }
                }

            }
		}
	}
}

pub fn fcommand( input: &str, arguments: Vec<String>, stdin_data: Vec<&str>) -> SService {
    let mut init1 = SService {
      stdout: String::new(),
      stderr: String::new(),
    };

    // Take the first, which is the binary to execute
    let binn = {
        if input.chars().next().unwrap() == '_' {
            let builtin = input.split('_').collect::<Vec<&str>>()[1];
            // For arguments we take the complete vector and then we remove the first position
            // which is the binary. Then we strip the '\n' at the end of string
            let b_arguments: String = {
                let mut buffer = String::new();
                for i in arguments {
                    if buffer.is_empty(){
                        buffer = i.clone();
                    } else {
                        buffer = buffer + " " + &i;
                    }
                }
                buffer
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
					Ok(d) => {
						d.split(',').collect::<Vec<&str>>()[0].to_string()
					},
					Err(_e) => {
                        init1.stderr = format!("Binary {:?} not found in PATH", input);
						return init1;
					},
				}
		} else {
				input.split(' ').map(|e| e.to_string()).collect::<Vec<String>>()[0].to_string()
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
        //proc.arg(j.trim()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

        // Stdio::inherit allows the child process to inherit the parent's file descriptors so that it can access the TTY terminal
        proc.arg(j.trim()).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    if stdin_data.is_empty(){
        // Spawn execute it
        let nproc = match proc.spawn(){
            Ok(d) => d,
            Err(e) => {
                init1.stderr = format!("Binary failed to execute; {e}");
                return init1;
            },
        };
        // Wait_with_output method waits to command finishes and collect output, return Output struct
        let output = match nproc.wait_with_output(){
            Ok(d) => d,
            Err(e) => {
                init1.stderr = format!("Failed to execute; {e}");
                return init1;
            },
        };
        init1.stdout = match std::str::from_utf8(&(output.stdout).clone()){
            Ok(d) => d.to_string(),
            Err(_e) => String::from("Error taking stdout in buffer"),
        };
        init1.stderr = match std::str::from_utf8(&(output.stderr).clone()){
            Ok(d) => d.to_string(),
            Err(_e) => String::from("Error taking stderr in buffer"),
        };
    } else {
        // Spawn execute it
        let mut nproc = match proc.spawn(){
            Ok(d) => d,
            Err(e) => {
                init1.stderr = format!("Binary failed to execute; {e}");
                return init1;
            },
        };
        if let Some(ref mut nproc_stdin) = nproc.stdin {
            for i in stdin_data {
                // Shadowing to allow programs like grep work properly
                let mut i = i.to_string();
                i.push('\n');
                match nproc_stdin.write_all(i.as_bytes()){
                    Ok(_d) => (),
                    Err(_e) => eprintln!("Error to write stdin"),
                }
            }
        } else {
            eprintln!("Error taking child's stdin");
            return init1;
        }
        // Wait_with_output method waits to command finishes and collect output, return Output struct
        let output = match nproc.wait_with_output(){
            Ok(d) => d,
            Err(e) => {
                init1.stderr = format!("Failed to execute; {e}");
                return init1;
            },
        };
        init1.stdout = match std::str::from_utf8(&(output.stdout).clone()){
            Ok(d) => d.to_string(),
            Err(_e) => String::from("Error taking stdout in buffer"),
        };
        init1.stderr = match std::str::from_utf8(&(output.stderr).clone()){
            Ok(d) => d.to_string(),
            Err(_e) => String::from("Error taking stderr in buffer"),
        };
    }
    init1
}
