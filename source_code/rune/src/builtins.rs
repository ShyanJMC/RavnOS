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


// Here we use a const and not let because is a global variable
// As we know the size of each word we can use "&str" and then we specify the number
// of elements. This is because a const must have know size at compiling time.
const LBUILTINS: [&str; 4] = ["info", "env", "list", "$?"];
const RUNE_VERSION: &str = "v0.1.0";

// Builtins
// Are private for only be executed by rbuiltins

fn info() -> String {
    let os = {
        if cfg!(linux) {
            "Linux"
        } else if cfg!(freebsd){
            "FreeBSD"
        } else if cfg!(dragonfly) {
            "DragonFLY BSD"
        } else if cfg!(openbsd) {
            "OpenBSD"
        } else if cfg!(netbsd) {
            "NetBSD"
        } else {
            "Unknown"
        }
    };

    let user = {
        let mut varvalue: String = String::new();
        for (key, value) in std::env::vars(){
            if key == "USER" {
                varvalue = String::from( value);
                break;
            }
        }
        varvalue
    };

    format!(" Rune Shell version; {RUNE_VERSION}\n OS: {}\n User: {} ",os, user)
}

fn environmentvar() -> String {
    let mut buffer: Vec<String> = Vec::new();
    let mut buffer2: String = String::new();
    for (key, value) in std::env::vars(){
        let envv = format!("{}; {}\n", key, value);
        buffer.push( envv );
    }
    for data in buffer {
        buffer2 = buffer2 + &data.to_string();
    }
    buffer2
}



////////////////

// Check the builtin executing it
// The first argument is the command, the second the lbuilts list
// Returns Ok(d) with the stdout of builtin or Err(e) if doesn't match
pub fn rbuiltins(command: &str) -> Result<String,String> {
    let mut result = String::new();
    // We MUST use trim() because always there are some unwelcomme characters at the start/end
    let command = command.trim();

    if LBUILTINS.contains( &command ){
        if command == "info" {
            // "self" is needed because this is a module, not a binary
            result = self::info();
            Ok(result)
        } else if command == "env" {
            result = environmentvar();
            Ok(result)
        } else if command == "list" {
            result = format!(" Bultins, they are called with '_'; {{\n {:?}\n}}", LBUILTINS);
            Ok(result)
        } else {
            Err( "builtin not recognized".to_string() )
        }
    } else {
        Err( "not builtin found".to_string() )
    }
}
