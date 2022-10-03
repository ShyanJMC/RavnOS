// Stream configuration
// Configuration
pub struct SConfiguration {
    file: bool,
    stdin: bool,
}

// Configuration struct
// Each field determine if option is enabled or not.
// Show
pub struct ShowConfiguration {
    pub size: bool,
    pub datetime: bool,
    pub lines: bool,
    pub owner: bool,
    pub permission: bool,
    pub clean: bool,
    pub stdin: bool,
    pub proc: bool,
}

// Ls
pub struct LsConfiguration {
    pub verbose: bool,
    pub proc: bool,
    pub lines: bool,
    pub clean: bool,
}

// Trait for checkarguments and returns files names
pub trait RavnArguments {
    fn checkarguments_stream(&self, config: &mut SConfiguration) -> Vec<String>;
    fn checkarguments_ls(&self, config: &mut LsConfiguration) -> Vec<String>;
    fn checkarguments_show(&self, config: &mut ShowConfiguration) -> Vec<String>;
    fn checkarguments_help(&self, program: String) -> bool;
}

// Behaviour and methods
impl RavnArguments for Vec<String> {
    fn checkarguments_stream(&self, config: &mut SConfiguration) -> Vec<String> {
        let mut arguments = Vec::new();
        for indexs in self {
            if indexs == "-f" {
                config.file = true;
            } else if indexs == "-i" {
                config.stdin = true;
            }

            //  "chars" method breaks input in individual chars
            //  "next" method will do start to position zero, which we need to know if start with
            //  "-" which indicate that is an option.
            //  Also I added for paths ( / ).
            //  Remember "unwrap" method extract X from Some(X) or Err(X) and/or from "Option<X>"
            //  also.
            let pzero = indexs.chars().next().unwrap();
            if pzero.to_string() != *"-".to_string() {
                arguments.push(indexs.clone());
            }
        }
        // Returns vec string.
        arguments
    }

    // self argument is he Vec<String> variable with arguments.
    // "config" is the configuration struct variable.
    fn checkarguments_show(&self,config: &mut ShowConfiguration) -> Vec<String> {
        let mut files = Vec::new();
        for indexs in self {
            if indexs == "-c" {
                config.clean = true;
            } else if indexs == "--stdin" {
                config.stdin = true;
            } else if indexs == "-s" {
                config.size = true;
            } else if indexs == "-d" {
                config.datetime = true;
            } else if indexs == "-l" {
                config.lines = true;
            } else if indexs == "-o" {
                config.owner = true;
            } else if indexs == "-p" {
                config.permission = true;
            } else if indexs == "--proc" {
                config.proc = true;
            }
        
            //  "chars" method breaks input in individual chars
            //  "next" method will do start to position zero, which we need to know if start with
            //  "-" which indicate that is an option.
            //  Also I added for paths ( / ).
            //  Remember "unwrap" method extract X from Some(X) or Err(X) and/or from "Option<X>"
            //  also.
            let pzero = indexs.chars().next().unwrap();
            if pzero.to_string() != *"-".to_string() && !config.stdin {
                files.push(indexs.clone());
            }
        }
        // Returns vec string.
        files
    }

    fn checkarguments_ls(&self, config: &mut LsConfiguration) -> Vec<String> {
        let mut dirs = Vec::new();
        for indexs in self {
            if indexs == "-v" {
                config.verbose = true;
            } else if indexs == "--proc" {
                config.proc = true;
            } else if indexs == "-l" {
                config.lines = true;
            } else if indexs == "-c" {
                config.clean = true;
            }
            //  "chars" method breaks input in individual chars
            //  "next" method will do start to position zero, which we need to know if start with
            //  "-" which indicate that is an option.
            //  Also I added for paths ( / ).
            //  Remember "unwrap" method extract X from Some(X) or Err(X) and/or from "Option<X>"
            //  also.
            let pzero = indexs.chars().next().unwrap();
            if pzero.to_string() != *"-".to_string() {
                dirs.push(indexs.clone());
            }
        }
        // Returns vec string.
        dirs
    }

            


    // self argument is he Vec<String> variable with arguments.
    // "config" is the configuration struct variable.
    // Check if some argument is "-h" or "--help"
    fn checkarguments_help(&self, program: String) -> bool {
        let mut help = false;
        // If you ask your self why I didn't put this in the "for" loop;
        // you can not assign a value to something that do not exist, basically
        // if is empty skip "for" loop.
        if self.is_empty() {
            help = true;
        }
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
            -d      : show datetime format.
            -l      : show file's data lines.
            -o      : show owner.
            -p      : show file permissions.
            -c      : clean verbose to show only file's content.
            --proc  : show the system's processes. Only in Unix systems.
            --stdin : read from standard input in addition of 'file n'.
            ".to_string();
            eprintln!("{}", var1);
            
            } else if program == "ls" {
                 let var1= "Usage;
            [option] [directory 1] [directory n]

            Options:
            --------
            -l      : show directory's files and directories number.
            -v      : show owner, permissions, datetime format and size (in bytes).
            -c      : clean verbose to show only directory's content.
            --proc  : show the system's processes trough /proc filesystem. Only in Unix systems.
            ".to_string();
            eprintln!("{}",var1);
            
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

            ".to_string();
            eprintln!("{}", var1);
            }
            help
        } else {
            help
        }
    }
}
