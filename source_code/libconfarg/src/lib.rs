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
    pub size: bool,
    pub datetime: bool,
    pub lines: bool,
    pub owner: bool,
    pub permission: bool,
    pub clean: bool,
    pub stdin: bool,
    pub proc: bool,
    pub hexa: bool,
    pub base64: bool,
    pub fbase64: bool,
    pub words: bool,
    pub env: bool,
    pub date: bool,
    pub diff: bool,
    pub systeminfo: bool,
    pub which: bool,
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
                    }
                }
            }

            "show" => {
                for indexs in self {
                    if indexs == "-c" {
                        options.push("clean");
                    } else if indexs == "--stdin" {
                        options.push("stdin");
                    } else if indexs == "-s" {
                        options.push("size");
                    } else if indexs == "-d" {
                        options.push("datetime");
                    } else if indexs == "-l" {
                        options.push("lines")
                    } else if indexs == "-o" {
                        options.push("owner");
                    } else if indexs == "-p" {
                        options.push("permission");
                    } else if indexs == "--proc" {
                        options.push("proc");
                    } else if indexs == "-w" {
                        options.push("words");
                    } else if indexs == "--hexa" {
                        options.push("hexa");
                    } else if indexs == "-e" {
                        options.push("environment");
                    } else if indexs == "--date" {
                        options.push("date");
                    } else if indexs == "--diff" {
                        options.push("diff");
                    } else if indexs == "--info" {
                        options.push("systeminfo");
                    } else if indexs == "--base64" {
                        options.push("base64");
                    } else if indexs == "--which" {
                        options.push("which");
                    } else if indexs == "--from-base64" {
                        options.push("from_base64");
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
            -s      : show file size in bytes .
            -d      : show datetime format (in UTC).
            -l      : show file's lines.
            -o      : show owner.
            -p      : show file permissions.
            -c      : clean verbose to show only file's content.
            -w      : show file's words.
            -e      : show ENV environment variable value.
            --proc  : show the system's processes. Only in Unix systems.
            --stdin : read from standard input in addition of 'file n'.
            --hexa  : show the file's content in hexa. For binary and text.
            --base64: show the file's content in base64. Only for text files right now.
            --date  : show current date based in Unix Epoch.
            --diff  : show the differences of second file with respect to first.
            --info  : show basic information about the operative system.
            --which : show where is located the binary provided as argument
            --from-base64 [stdin] [file] : convert base64 encoding to binary, saving to file
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
