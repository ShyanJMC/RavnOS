// Standar libraries

//// Environment lib
use std::env;
//// Filesystem lib
use std::fs;
//// String slice lib
// use std::str;
//// For Unix platform
use std::os::unix::fs::MetadataExt;
//// I/O lib
use std::io;
// Process lib
use std::process;


// RavnOS libraries
use libconfarg::RavnArguments;
use libstream::getprocs;

fn main() {
    // env::args() takes program's arguments (the first is always the self binary).
    // collect() takes arguments and returns in tuple
    let mut arguments: Vec<String> = env::args().collect();

    // Remove the binary name from vector (remove the data from index zero).
    arguments.remove(0);

    // Init the configuration as clean
    let mut config = libconfarg::ShowConfiguration {
        size: false,
        datetime: false,
        lines: false,
        owner: false,
        permission: false,
        clean: false,
        stdin: false,
        proc: false,
    };

    if arguments.checkarguments_help( "show".to_string() ) {
        process::exit(0);
    }

    // The vec<String> return with files index is stored in "archives" variable.
    // The method is from RavnArguments trait.
    let archives: Vec<String> = arguments.checkarguments_show(&mut config);

    // Stdinput
    if config.stdin {
        // Buffer variable to store returns
        let mut buffer = String::new();
        // To work with stdin
        match io::stdin().read_line(&mut buffer) {
            Ok(_i) => println!("{buffer}"),
            Err(j) => println!("error; {j}"),
        }
    }

    // Showing procs
    if config.proc {
        getprocs();
    }

    // Opening files and showing them
    for names in &archives {
        // " if X = false " is equal to; " if !X "
        // " if X = true " is equal to; " if X "
        if !config.clean {
            println!("File Name: {names}");
            if config.size {
                println!(
                    "Size (bytes): {:?}",
                    fs::metadata(&names).expect("Error reading matadata").len()
                );
            }

            if config.datetime {
                println!(
                    "Modified (DateTime): {:?}\nAccessed: {:?}\nCreated: {:?}",
                    fs::metadata(&names)
                        .expect("Error reading matadata")
                        .modified()
                        .unwrap(),
                    fs::metadata(&names)
                        .expect("Error reading matadata")
                        .accessed()
                        .unwrap(),
                    fs::metadata(&names)
                        .expect("Error reading matadata")
                        .created()
                        .unwrap()
                );
            }

            if config.lines {
                println!(
                    "Lines: {:?}",
                    fs::read_to_string(names)
                        .expect("Error reading file.")
                        .lines()
                        .count()
                );
            }

            if config.owner {
                // Check if the target OS is some of Unix family.
                if cfg!(target_os = "linux")
                    || cfg!(target_os = "freebsd")
                    || cfg!(target_os = "dragonfly")
                    || cfg!(target_os = "openbsd")
                    || cfg!(target_os = "netbsd")
                    || cfg!(target_family = "unix")
                {
                    println!(
                        "Owner: {:?}",
                        fs::metadata(&names).expect("Error reading metadata").uid()
                    );
                } else {
                    println!("Owner: Not supported because platform is not detected as Unix.");
                }
            }

            if config.permission {
                println!(
                    "Permission: {:?}",
                    fs::metadata(&names)
                        .expect("Error reading matadata")
                        .permissions()
                );
            }
        }

        // Opening file and reading it as string.
        let fstring = fs::read_to_string(names).expect("Error reading file.");
        // Using by reference to not take "len" the ownership of "archives".

        if !config.clean {
            println!("=================\n");
        }
        if archives.len() > 1 && !config.proc {
            println!(
                "{}\n\n-------------------------------------------------\n",
                fstring
            );
        } else {
            println!("{}", fstring);
        }
    }
}
