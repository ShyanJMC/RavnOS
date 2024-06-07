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


// HashMap lib
use std::collections::HashMap;
// File lib
use std::fs::{File,OpenOptions};
use std::io::prelude::*;
// Path lib
use std::path::Path;
// Process lib
use std::process;

use libstream::{Colors,Stream};
use libcommand;

// Version constant
const HVERSION: &str = "0.1.1";

fn main() {
    let color = Colors::new();
    let mut threads_services = Vec::new();

    println!("Huginn sysinit version {HVERSION}");

    // Create huginn path configuration if do not exists
    if !( Path::new("/etc/huginn/").try_exists().unwrap() ) {
        match std::fs::create_dir_all("/etc/huginn/".to_string()){
            Ok(_d) => println!("[INFO]\tCreated huginn path configuration; /etc/huginn"),
            Err(e) =>  eprintln!("[ERR]\tError creating huginn path coniguration (/etc/huginn); {e}"),
        };

    } else if !( Path::new("/etc/huginn/services").try_exists().unwrap() ) {
        // Create service file is not exists
        match OpenOptions::new().create(true).append(true).open( "/etc/huginn/services" ) {
    		Ok(_d) => println!("[INFO]\tCreated huginn file configuration; /etc/huginn/services"),
    		Err(e) => {
    			eprintln!("[ERR]\tError creating /etc/huginn/services file: {e}");
                process::exit(1);
    		},
    	};
    } else {}

    // Takes service's name and binary
    // "readkey" returns a HashMap with <Service_name, binary_arguments>
    let hservices: HashMap<String,HashMap<String, String>> = {
        let mut fservices = String::new();
        File::open("/etc/huginn/services").expect("Error opening configuration service.").read_to_string(&mut fservices).expect("Error reading configuration service.");
        let buff = fservices.readkey();
        let mut vreturn: HashMap<String,HashMap<String, String>> = Default::default();
        for (serv,data) in buff{
            let buff2 = data.readconfig();
            vreturn.insert(serv,buff2);
        }
        vreturn
    };
    println!("Starting services (/etc/huginn/services)");
    for (serv,data) in &hservices {
        let serv = serv.trim().to_string();
        let binary = match data.get("binary"){
            Some(data) => data.to_string(),
            None => {
                eprintln!("{}[ERR]{serv}\n\tFailing detecting binary{}", color.red, color.reset);
                "".to_string()
            },
        };
        let arguments = match data.get("arguments"){
            Some(data) => data.to_string(),
            None => "".to_string(),
        };

        let command = libcommand::thread_command(serv, binary, arguments);
        threads_services.push(command);
    }

    for handles in threads_services {
        match handles.join(){
            Ok(_d) => (),
            Err(_e) => {
                eprintln!("{}[ERR]Failing awaiting for thread!{}", color.red, color.reset);
                ()
            },
        };
    }

}
