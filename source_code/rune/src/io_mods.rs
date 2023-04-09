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
use std::io::Read;


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
