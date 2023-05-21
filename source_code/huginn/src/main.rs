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
use std::io::{self,prelude::*};
// Path lib
use std::path::Path;
// Process lib
use std::process::{self,ChildStdout,ChildStderr,ExitStatus,Stdio};

use libstream::{Colors,Stream};
use libcommand;

// Version constant
const HVERSION: &str = "0.1.0";

fn main() {
    let color = Colors::new();
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
    		Ok(_d) => println!("[INFO]\tCreated huginn file configuration; /etc/huginn/service"),
    		Err(e) => {
    			eprintln!("[ERR]\tError creating /etc/huginn/services file: {e}");
                process::exit(1);
    		},
    	};
    } else {}

    let mut fservices = String::new();
    File::open("/etc/huginn/services").expect("Error opening configuration service.").read_to_string(&mut fservices).expect("Error reading configuration service.");

    // Takes service's name and binary
    let hservices: HashMap<String,String> = fservices.readkey();
    println!("Starting services (/etc/huginn/services)");
    // Take name and set ID to zero
    let mut h_s_service: HashMap<String,i64> = {
        let mut buff: HashMap<String,i64> = HashMap::new();
        for (serv,bin) in &hservices {
            let serv = serv.trim().to_string();
            buff.insert(serv,0);
        }
        buff
    };

    let mut threads_services = Vec::new();

    for (serv,bin) in &hservices {
        let serv = serv.trim();
        let binary = bin.trim();

        let command = libcommand::thread_command(h_s_service.clone(), serv.to_string(), binary.to_string());
        threads_services.push(command);
    }

    for handles in threads_services {
        match handles.join(){
            Ok(_d) => (),
            Err(e) => {
                eprintln!("{}[ERR]Failing awaiting for thread!{}", color.red, color.reset);
                ()
            },
        };
    }

}
