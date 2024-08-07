//! This file is part of RavnOS.
//!
//! RavnOS is free software:
//! you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation,
//! either version 3 of the License, or (at your option) any later version.
//!
//! RavnOS is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
//! without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License along with RavnOS. If not, see <https://www.gnu.org/licenses/>

//!
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022

// Note; take under consideration that because String dereference to &str, is better use in
// functions signatures &str instead String. Unless the variable is an String in another point of
// the program.

//! This lib contains the methods to check RavnOS's arguments in each program

/// Configuration struct
/// Each field determine if option is enabled or not.
/// Show configuration struct
pub struct ShowConfiguration {
    pub clean: bool,
    pub stdin: bool,
    pub hexa: bool,
}

/// Ls configuration struct
pub struct LsConfiguration {
    pub verbose: bool,
    pub proc: bool,
    pub lines: bool,
    pub clean: bool,
}

/// Search configuration struct
pub struct SearchConfiguration {
    pub file: bool,
    pub binary: bool,
    pub directory: bool,
    pub environment: bool,
    pub processes: bool,
    pub recursive: bool,
    pub input: bool,
    pub ravnkey: bool,
}

/// Trait for checkarguments and returns files names or show help
pub trait RavnArguments {
    fn check_arguments(&self, soft: &str, options: &mut Vec<&str>) -> Vec<String>;
    fn checkarguments_help(&self, program: &str) -> bool;
}

/// Behaviour and methods
impl RavnArguments for Vec<String> {
    /// self is the Vec<String> with the arguments.
    /// options is the vec &str wich will contains the options based in the soft name.
    /// soft is the variable with programs name.
    ///
    /// The returns is a string vector with each argument without the options (that is
    /// stored into config).
    fn check_arguments(&self, soft: &str, options: &mut Vec<&str>) -> Vec<String> {
        let mut arguments = Vec::new();
        // Match the variable value to some word, as String dereference to &str is better
        // use &str directly.
        match soft {
            "edit" => {
                for indexs in self {
                    if indexs == "-f" {
                        options.push("file");
                    } else if indexs == "-i" {
                        options.push("stdin");
                    } else {
                        options.push("");
                    }
                }
            }

            "show" => {
                for indexs in self {
                    if indexs == "-c" {
                        options.push("clean");
                    } else if indexs == "--stdin" {
                        options.push("stdin");
                    } else if indexs == "--hexa" {
                        options.push("hexa");
                    } else if indexs == "--diff" {
                        options.push("diff");
                    } else {
                        options.push("");
                    }
                }
            }
            "ls" => {
                for indexs in self {
                    if indexs == "-v" {
                        options.push("verbose");
                    } else if indexs == "--proc" {
                        options.push("proc");
                    } else if indexs == "-l" {
                        options.push("lines");
                    } else if indexs == "-c" {
                        options.push("clean");
                    } else {
                        options.push("");
                    }
                }
            }
            "search" => {
                for indexs in self {
                    if indexs == "-f" {
                        options.push("file");
                    } else if indexs == "-d" {
                        options.push("directory");
                    } else if indexs == "-e" {
                        options.push("environment");
                    } else if indexs == "-p" {
                        options.push("proc");
                    } else if indexs == "-r" {
                        options.push("recursive");
                    } else if indexs == "-s" {
                        options.push("input");
                    } else if indexs == "-rk" {
                        options.push("ravnkey");
                    } else if indexs == "-b" {
                        options.push("binary");
                    } else {
                        options.push("");
                    }
                }
            }

            _ => std::process::exit(1),
        }

        for indexs in self {
            //  "chars" method breaks input in individual chars
            //  "next" method will do start to position zero, which we need to know if start with
            //  "-" which indicate that is an option.
            //  Also I added for paths ( / ).
            //  Remember "unwrap" method extract X from Some(X) or Err(X) and/or from "Option<X>"
            //  also.
            let pzero = match indexs.chars().next() {
                Some(d) => d,
                None => ' ',
            };
            if pzero.to_string() != *"-".to_string() {
                arguments.push(indexs.clone());
            }
        }
        // Returns vec string.
        arguments
    }

    /// check if some arguments is the help
    fn checkarguments_help(&self, program: &str) -> bool {
        let mut help = false;

        for indexs in self {
            if indexs == "-h" || indexs == "--help" {
                help = true;
            }
        }
        if help {
            if program == "show" {
                // Is not needed use format! macro because I'm just formatting a literal text, not
                // with variables.
                let var1 = "Usage;
            [option] [file 1] [file n]

            Options:
            --------
            -c      : clean verbose to show only file's content.
            --stdin : read from standard input in addition of 'file n'.
            --hexa  : show file's content in hexadecimal.
            "
                .to_string();
                eprintln!("{}", var1);
            } else if program == "ls" {
                let var1 = "Usage;
            [option] [directory 1] [directory n]

            Options:
            --------
            -l      : show directory's files and directories number.
            -v      : show owner, permissions, datetime format and size (in bytes).
            -c      : clean verbose to show only directory's content.
            --proc  : show the system's processes trough /proc filesystem. Only in Unix systems.
            "
                .to_string();
                eprintln!("{}", var1);
            } else if program == "edit" {
                // Is not needed use format! macro because I'm just formatting a literal text, not
                // with variables.
                let var1 = "Usage;
            [option] [stream]

            Options:
            --------
            -f      : edit file
            -i      : edit input
            [origin text] [dest text]

            "
                .to_string();
                eprintln!("{}", var1);
            } else if program == "search" {
                let var1 = "Usage;
            [option] [String] [path_file_key_or_environment_variable]

            Options:
            --------
            -b      : search inside binary.
            -d      : search the string in directories' name.
            -e      : search the string in environment variables data.
            -f      : search the string inside file.
            -p      : search the string in system processes.
            -r      : search recursively the string in directories' name and file's data.
            -s      : search the string in stdin
            -rk		: search the data from key. Take the data and key with sintax; [key] { [data] }
            "
                .to_string();
                eprintln!("{}", var1);
            }
            help
        } else {
            help
        }
    }
}
