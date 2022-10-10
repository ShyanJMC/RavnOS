// Standar libraries

//// Environment lib
use std::env;
//// Filesystem lib
use std::fs;
//// String slice lib
// use std::str;
//// For Unix platform
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

//// I/O lib
use std::io;
// Process lib
use std::process::{self, Command};

// RavnOS libraries
use libconfarg::RavnArguments;
use libstream::{getprocs, OutputMode};

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

    if arguments.checkarguments_help("show".to_string()) {
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
        let meta = fs::metadata(&names).unwrap();
        // " if X = false " is equal to; " if !X "
        // " if X = true " is equal to; " if X "
        if !config.clean {
            println!("File Name: {names}");
            if config.size {
                println!("Size (bytes): {:?}", meta.len());
            }

            if config.datetime {
                println!(
                    "Modified (EPoch): {:?}\nAccessed (EPoch): {:?}\nCreated (EPoch): {:?}",
                    meta.mtime(),
                    meta.atime(),
                    meta.ctime()
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
                    // Call "id" command to detect who is ID number owner.
                    let ownerout = Command::new("/usr/bin/id")
                        .arg(meta.uid().to_string())
                        .output()
                        .unwrap();
                    // When you use "output" method, the stdout of command will be stored in
                    // "stdout" field. But, is stored as u8, and needs to be processed as utf8.
                    println!(
                        "Owner: {}",
                        std::str::from_utf8(&ownerout.stdout)
                            .unwrap()
                            .strip_suffix("\n")
                            .unwrap()
                    );
                } else {
                    println!("Owner: Not supported because platform is not detected as Unix.");
                }
            }

            if config.permission {
                // Permissions method by default will return in bits, if you want the octal chmod
                // syntax need to use ".mode()".
                // As Octal is not a type by it self, we need use "format!" macro to convert it in
                // octal mode, the return is a String.
                println!("{:o}", meta.permissions().mode());

                println!(
                    "Permission: {:?}",
                    format!("{:o}", meta.permissions().mode()).permission_to_human()
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
