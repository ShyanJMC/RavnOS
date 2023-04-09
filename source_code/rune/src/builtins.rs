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

// Here we use a const and not let because is a global variable
// As we know the size of each word we can use "&str" and then we specify the number
// of elements. This is because a const must have know size at compiling time.
const LBUILTINS: [&str; 9] = ["cd", "clear", "env", "exit", "history", "home", "info", "list", "$?"];
const RUNE_VERSION: &str = "v0.9.0";

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

fn cd(path: String) -> () {
    let buff = path.trim();
    let npath = Path::new( &buff );
    env::set_current_dir(&npath).expect("Failing setting the new working path");
}

fn clear() {
    // \x1B[ : ASCII scape character and start control secuence
    // \x1B[2J: Clears the entire screen
    // \x1B[<n>;<m>H: Moves the cursor to row <n> and column <m>
    // \x1B[<n>A: Moves the cursor <n> lines up
    // \x1B[<n>B: Moves the cursor <n> lines down
    // \x1B[<n>C: Moves the cursor <n> columns right
    // \x1B[<n>D: Moves the cursor <n> columns left
    // \x1B[K: Clears from the cursor position to the end of the line
    // \x1B[<n>J: Clears the screen from the cursor position to the end of the screen if <n> is 0, from the beginning of the screen to the cursor position if <n> is 1, and clears the entire screen if <n> is 2
    // \x1B[<n>m: Sets the text style. <n> can be 0 (sets style to normal), 1 (sets style to bold), 2 (sets style to dim), 3 (sets style to italic), 4 (sets style to underline), 5 (sets style to blinking), 7 (inverts the foreground and background colors), 8 (hides the text), 22 (disables bold or dim style), 23 (disables italic style), 24 (disables underline style), 25 (disables blinking style), 27 (disables color inversion) and 28 (shows hidden text).
    // \x07: Emits a beep or alert sound
    print!("\x1B[1;1H\x1B[2J");
}


////////////////

// Check the builtin executing it
// The first argument is the command, the second the lbuilts list
// Returns Ok(d) with the stdout of builtin or Err(e) if doesn't match
pub fn rbuiltins(command: &str, b_arguments: String) -> Result<String,&'static str> {
    let result: String;
    // We MUST use trim() because always there are some unwelcomme characters at the start/end
    let command = command.trim();

    if LBUILTINS.contains( &command ){
        if command == "info" {
            // "self" is needed because this is a module, not a binary
            result = self::info();
            Ok(result)
        } else if command == "cd" {
            self::cd(b_arguments);
            Ok(" ".to_string())
        } else if command == "env" {
            result = environmentvar();
            Ok(result)
        } else if command == "list" {
            result = format!(" Bultins, they are called with '_'; {{\n {:?}\n}}", LBUILTINS);
            Ok(result)
        } else if command == "clear" {
            self::clear();
            Ok( " ".to_string() )
        } else {
            Err( "builtin not recognized" )
        }
    } else {
        Err( "not builtin found" )
    }
}
