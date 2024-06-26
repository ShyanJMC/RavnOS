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

use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

// For epoch_to_human()
use libstream::Epoch;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn user_history(command: &String, vhistory: &Vec<String>, rune_history: &mut File, vhistory_map: &mut HashMap<usize,String>) -> Result<(),()>{
    if !command.is_empty() {
        let hist_command = {
            let unix_date = match SystemTime::now().duration_since(UNIX_EPOCH){
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error getting duration since UNIX_EPOCH; \n {e}");
                    return Err(());
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
                return Err(());
            },
        }

        match rune_history.write_all(b"\n"){
            Ok(_d) => (),
            Err(_e) => {
                eprintln!("Error saving command to history file");
                return Err(());
            },
        }

        let mut temp_buff: usize = 0;
            for i in vhistory {
                vhistory_map.insert(temp_buff, i.clone());
                temp_buff += 1;
            }
        }
        Ok(())
}
