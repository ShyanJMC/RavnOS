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

use std::env;
use std::path::Path;
use std::fs::{self,File};
use std::io::{self,Read};
use std::collections::HashMap;

// Threads and comunication sync
use std::thread;
use std::sync::mpsc;

// Read the input byte per byte
pub fn read_input() -> Result<String,String> {

    let entry: String;

    //  Pipeline for thread communicaton of u8 type
    // sc = send channel ( with method; send([variable_x]); )
    // rc = recive channel ( with method; recv(); )
    // Remember; in Rust can be only one sender but many recivers
    // The first channel is for thread's information, the second is the results in each
    let (sc,rc) = mpsc::channel();
    let (rsc, rrc) = mpsc::channel();

    // Read the stdin
    // Rust stdlib is sync, not async because of that for now I can't do a real time shell, for now
    let thread_shell_input = thread::Builder::new().name("Shell Input".to_string() ).spawn( move || {
            loop {
                let mut input = io::stdin();

                // Type 0 (u8) of 3 elements
                // ASCII = 1 byte
                // UTF-8 = 1 byte
                // UTF-16 = 2 bytes
                // ASCII scapes = 3 bytes
                let mut buffer = [0; 1];

                match input.read_exact(&mut buffer){
                    Ok(_d) => (),
                    Err(e) => {
                        eprintln!("Error in thread_shell_input thread reading stdin; \n {}", e);
                        ()
                    },
                }

                // Send buffer variable to channel/pipeline
                if buffer > [0] {
                    let uresult = match sc.send( buffer ) {
                        Ok(_d) => Ok( "Data sent to thread channel".to_string() ),
                        Err(_e) => Err( "Fail comunicating trough threads".to_string() ),
                    };
                    drop(uresult);
                }
                if buffer == [b'\n'] {
                    break;
                }


            }
        }
    ).unwrap();

    let thread_shell_check = thread::Builder::new().name( "Shell checker".to_string() ).spawn( move || {
            let mut command = String::new();
            let mut ascii_input = String::new();
            let mut buffer = [0; 3];

            loop {
                match rc.recv() {
                    Ok(d) => {

                        if d == [b'\x1b'] {
                            buffer[0] = 1;
                        } else if buffer[0] == 1 && d == [b'['] {
                            buffer[1] = 1;
                        } else if buffer[0] == 1 && buffer[1] == 1 && d == [b'A'] {
                            ascii_input = "up_arrow".to_string();
                        } else if buffer[0] == 1 && buffer[1] == 1 && d == [b'B'] {
                            ascii_input = "down_arrow".to_string();
                        } else if buffer[0] == 1 && buffer[1] == 1 && d == [b'C'] {
                            ascii_input = "right_arrow".to_string();
                        } else if buffer[0] == 1 && buffer[1] == 1 && d == [b'D'] {
                            ascii_input = "left_arrow".to_string();

                            // If is "Enter" which is really a new line
                            // Because depending of kernel can change the ASCII I use many checks for the same
                        } else if d[0] == b'\n' || d[0] == b'\r' {
                            // Breaks the loop, and the thread
                            break;
                        // If is not ASCII scape or newline is a character
                        } else {
                            command.push( char::from(d[0]) );
                        }

                    },
                    Err(_e) => continue,
                }

            }

            if ascii_input.is_empty() {
                match rsc.send(command) {
                    Ok(_d) => Ok( "Data sent to thread channel".to_string() ),
                    Err(_e) => Err( "Fail comunicating trough threads".to_string() ),
                }
            } else {
                match rsc.send(ascii_input) {
                    Ok(_d) => Ok( "Data sent to thread channel".to_string() ),
                    Err(_e) => Err( "Fail comunicating trough threads".to_string() ),
                }
            }

    }
    ).unwrap();


    match thread_shell_check.join() {
        Ok(_d) => (),
        Err(e) => {
            eprintln!("Error executing thread_shell_check; \n {:?}", e);
            ()
        },
    }

    match thread_shell_input.join() {
        Ok(_d) => (),
        Err(e) => {
            eprintln!("Error executing thread_shell_input; \n {:?}", e);
            ()
        },
    }

    entry = match rrc.recv() {
        Ok(d) => d,
        Err(e) => e.to_string(),
    };

    Ok( entry )

}


// Get user home from /etc/passwd file
pub fn get_user_home() -> String {
    let mut varvalue: String = String::new();
    for (key, value) in env::vars(){
        if key == "USER" {
            varvalue = String::from( value.trim() );
            break;
        }
    }
    let mut ffile = File::open("/etc/passwd").expect("Fail opening /etc/passwd file");
    let mut buffer: String = String::new();
    let mut buffer2 = Vec::new();
    let home: String;

    ffile.read_to_string( &mut buffer ).expect("Fail reading /etc/passwd file");
    drop(ffile);

    let vtemp = buffer.clone();
    drop(buffer);
    for ddata in vtemp.lines(){
        if ddata.contains(&varvalue) {
            buffer2 = ddata.split(':').collect::<Vec<&str>>();
        }
    }

    home = buffer2[5].to_string();
    home
}

pub fn get_history() -> Result<Vec<String>,()> {
    let home = self::get_user_home();
    let ravnos_home = home.clone() + &"/.ravnos/".to_string();
    let rune_history = ravnos_home.clone() + &"rune_history";

    // Create the dir if not exist
    match fs::create_dir_all( Path::new(&ravnos_home) ) {
        Ok(_d) => {},
        Err(_e) => eprintln!("{_e}"),
    }
    if !Path::new(&rune_history).exists(){
        match fs::File::create(rune_history.clone() ) {
            Ok(_d) => println!("Created rune history file; {rune_history}"),
            Err(_e) => eprintln!("{_e}"),
        }
    }

    drop(home);
    drop(ravnos_home);

    let mut rune_history = File::open(rune_history).expect("Fail to open rune history, check ~/.ravnos/rune_history file");
    let mut buffer = String::new();
    let mut history: Vec<String> = Vec::new();

    rune_history.read_to_string( &mut buffer ).expect("Fail reading rune history, check ~/.ravnos/rune_history file");
    for ddata in buffer.lines(){
        history.push( ddata.to_string() );
    }

    Ok(history)

}

pub fn aliases() -> HashMap<String,String> {
	let rune_aliases = self::get_user_home() + &"/.ravnos/" + &"rune_alias";
	// Create the dir if not exist
	match fs::create_dir_all( Path::new(&(self::get_user_home() + &"/.ravnos/")) ) {
		Ok(_d) => {},
	    Err(_e) => eprintln!("{_e}"),
	}
	if !Path::new(&rune_aliases).exists(){
		match fs::File::create(rune_aliases.clone() ) {
			Ok(_d) => {},
			Err(_e) => eprintln!("{_e}"),
		}
	}

	let mut ffile = File::open(rune_aliases).expect("Fail to open rune alias, check ~/.ravnos/rune_alias file");
	let mut buffer = String::new();
	let mut lalias: HashMap<String,String> = HashMap::new();

	ffile.read_to_string(&mut buffer).expect("Fail to open rune alias, check ~/.ravnos/rune_alias file");
	for ddata in buffer.lines(){
		let list = ddata.split("=").map(|e| e.trim()).collect::<Vec<&str>>();
		lalias.insert(list[0].to_string(),list[1].to_string());
	}
	lalias
	
}
