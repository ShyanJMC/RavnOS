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

// Standar libraries

//// Environment lib
use std::env;
//// Filesystem lib
use std::fs::{self,File};
//// String slice lib
// use std::str;
//// For Unix platform
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

//// I/O lib
use std::io;
use std::io::Read;

// Process lib
use std::process::{self, Command};

// Time lib
use std::time::SystemTime;

// HashMaps lib
use std::collections::HashMap;

// Path lib
use std::path::Path;

// RavnOS libraries
extern crate libconfarg;
extern crate libfile;
extern crate libstream;

use libconfarg::RavnArguments;
use libfile::{RavnSizeFile, RavnFile, which};
use libstream::{getprocs, Stream, file_filter,Epoch};

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
        hexa: false,
        base64: false,
        words: false,
        env: false,
        date: false,
        diff: false,
        which: false,
        systeminfo: false,
    };

    if arguments.checkarguments_help("show") {
        process::exit(0);
    }

    // The vec<String> return with files index is stored in "archives" variable.
    // The method is from RavnArguments trait.
    let mut options: Vec<&str> = Vec::new();
    let archives: Vec<String> = arguments.check_arguments("show", &mut options);

    for confs in options {
        if confs == "size" {
            config.size = true;
        } else if confs == "datetime" {
            config.datetime = true;
        } else if confs == "lines" {
            config.lines = true;
        } else if confs == "owner" {
            config.owner = true;
        } else if confs == "permission" {
            config.permission = true;
        } else if confs == "clean" {
            config.clean = true;
        } else if confs == "stdin" {
            config.stdin = true;
        } else if confs == "proc" {
            config.proc = true;
        } else if confs == "hexa" {
            config.hexa = true;
        } else if confs == "words" {
            config.words = true;
        } else if confs == "environment" {
            config.env = true;
        } else if confs == "date" {
        	config.date = true;
        } else if confs == "diff" {
        	config.diff = true;
        } else if confs == "systeminfo" {
        	config.systeminfo = true;
        } else if confs == "base64" {
            config.base64 = true;
        } else if confs == "which" {
            config.which = true;
        }
    }

    // Stdinput
    if config.stdin {
        // Buffer variable to store returns
        let mut buffer = String::new();
        // To work with stdin
        // Match takes the read_line output, if is
        // Ok(_i) will print the buffer variable, but if
        // is Err(j) will print "j" (the error per se).
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_i) => println!("stdin {{ {buffer} }}"),
            Err(j) => println!("error {{ {j} }}"),
        }
    }

    // Systen imformation

    if config.systeminfo {

    	// Here I use a lot of shadowing, is just to save a bit of memory

    	let mut hmap = HashMap::new();

    	// Check if file exists
    	let fileinfo = if Path::new("/etc/os-release").exists() {
    		"/etc/os-release".to_string()
    	} else {
    		"/var/lib/os-release".to_string()
    	};

    	// Check if file exists
    	let host_name = if Path::new("/etc/hostname").exists() {
    		let mut file = File::open("/etc/hostname").unwrap();
    		let mut buffer = String::new();
    		file.read_to_string(&mut buffer).expect("Error reading file.");
    		// When is moved to heap memory deletes the \n and \r
    		buffer.pop();
    		buffer
    	} else {
    		"File /etc/hostname doesn't exists.".to_string()
    	};

    	hmap.insert("Hostname",host_name);

    	// boot id
    	let boot_id = if Path::new("/proc/sys/kernel/random/boot_id").exists(){
    		let mut file = File::open("/proc/sys/kernel/random/boot_id").unwrap();
    		let mut buffer = String::new();
    		file.read_to_string(&mut buffer).expect("Error reading file");
    		// When is moved to heap memory deletes the \n and \r
    		buffer.pop();
    		buffer
    	} else {
    		String::from("File /proc/sys/kernel/random/boot_id doesn't exist.")
    	};

    	hmap.insert("Boot ID", boot_id);

		// Read file and save it to buffer.
    	let os_name = file_filter( &fileinfo, "NAME".to_string() );
    	let mut os_name = os_name[0].split('=');
    	os_name.next();
    	let os_name = os_name.next().unwrap();

    	let os_pretty = file_filter( &fileinfo, "PRETTY_NAME".to_string() );
    	let mut os_pretty = os_pretty[0].split('=');
    	os_pretty.next();
    	let os_pretty = os_pretty.next().unwrap();

		// Insert a String as the key and the data as the hash.
		hmap.insert("Operative System",format!("{} ( {} )", os_pretty, os_name ) );

    	let os_url = file_filter( &fileinfo, "HOME_URL".to_string() );
    	let mut os_url = os_url[0].split('=');
    	os_url.next();
    	let os_url = os_url.next().unwrap();

    	let os_doc = file_filter( &fileinfo, "DOCUMENTATION_URL".to_string() );
    	let mut os_doc = os_doc[0].split('=');
    	os_doc.next();
    	let os_doc = os_doc.next().unwrap();

    	let os_legal = file_filter( &fileinfo, "PRIVACY_POLICY_URL".to_string() );
    	let mut os_legal = os_legal[0].split('=');
    	os_legal.next();
    	let os_legal = os_legal.next().unwrap();

		hmap.insert("Operative System information", format!("\n\tdocumentation: {}\n\turl: {}\n\tlegal: {}", os_doc, os_url, os_legal) );

		let machine_id = if Path::new("/etc/machine-id").exists() {
			let mut file = File::open("/etc/machine-id").unwrap();
			let mut buffer = String::new();
			file.read_to_string(&mut buffer).expect("Error reading file");
			buffer.pop();
			buffer
		} else {
			String::from("File /etc/machine-id doesn't exist.")
		};

		hmap.insert("Machine ID", machine_id);

		// Take CPU information from /proc/cpuinfo
		let cpuinfo = Path::new("/proc/cpuinfo").exists();

		// Message in case /proc/cpuinfo doesn't exists
		let cpuinfo_error = "The /proc/cpuinfo file doesn't exists. CPU information unavailable.";

		// Take the CPU model
    	let cpu: Vec<String> = if cpuinfo {
    		file_filter( &"/proc/cpuinfo".to_string(), "model name".to_string() )
    	} else {
    		cpuinfo_error.chars().map(|e| e.to_string() ).collect::<Vec<String>>()
    	};
    	let cpu = cpu[0].clone();

    	// Take the CPU threads number
    	// if the return is zero, means an error
    	let cputhread = if cpuinfo {
    		file_filter ( &"/proc/cpuinfo".to_string(), "processor".to_string() ).len()
    	} else {
    		0
    	};

    	hmap.insert("CPU Model", cpu);
    	hmap.insert("CPU Threads", format!("{cputhread}"));

    	// Take information about memory from /proc/meminfo
		let meminfo = Path::new("/proc/meminfo").exists();

		// As the information is stored in kB to transform into GB is needed
		// devide it.
		let memtotal = if meminfo {
			let temp = file_filter ( &"/proc/meminfo".to_string(), "MemTotal".to_string() );
			let mut temp = temp[0].split("       ");
			temp.next();
			temp.next().unwrap().to_string()

		} else {
			"Memory information not available".to_string()
		};

		hmap.insert("Memory", memtotal);

		// Kernel CMD Line
		// Check if file exists
		let cmdline = Path::new("/proc/cmdline").exists();

		// If exists read the file
		let kernelcmd = if cmdline {
			String::from_utf8(fs::read("/proc/cmdline").unwrap()).unwrap()
		} else {
			String::from("Kernel cmdline not available")
		};

		hmap.insert("Kernel CMD", kernelcmd);

		// Kernel version
		let kernel_version = if Path::new("/proc/version").exists() {
			let mut file = File::open("/proc/version").unwrap();
			let mut buffer = String::new();
			file.read_to_string(&mut buffer).expect("Error reading file");
			let mut buffer = buffer.split(" ");
			let kernel = buffer.next().unwrap();
			buffer.next();
			format!("{}: {}", kernel, buffer.next().unwrap() )
		} else {
			String::from("Can not read kernel version.")
		};

		hmap.insert("Kernel version", kernel_version);

		// "K" is the string key.
		// "V" is the data.
		// Print each one with the RavnOS [key] { [data]  } format.
		for (k,v) in hmap {
			println!("{k} {{ {v} }}\n");
		}

    }

    // Difference

    if config.diff {

    	// file 1
		let mut file1: File = File::open(&archives[0]).expect("Error opening file 1.");
    	let mut file1_buffer: String = String::new();
		file1.read_to_string(&mut file1_buffer).expect("Error reading file 1");

    	// file 2
    	let mut file2: File = File::open(&archives[1]).expect("Error opening file 2.");
    	let mut file2_buffer: String = String::new();
    	file2.read_to_string(&mut file2_buffer).expect("Error reading file 2.");

		let mut linen1: u64 = 0;
		let mut linen2: u64 = 0;
		let mut linebuffer = 1;


		let mut hmap1 = HashMap::new();
		let mut hmap2 = HashMap::new();

    	for ilines in file1_buffer.lines() {
    		linen1 += 1;
    		hmap1.insert(linen1, ilines);
    	}

    	for ilines2 in file2_buffer.lines() {
    		linen2 += 1;
    		hmap2.insert(linen2, ilines2);
    	}

    	while linebuffer <= linen2 {

    		if !hmap1.contains_key( &linebuffer ) {
    			println!("ln {linebuffer} +{{ {} }}", hmap2.get(&linebuffer).unwrap() );
    		} else {
    			if hmap1.get(&linebuffer) != hmap2.get(&linebuffer) {
    				println!("ln {linebuffer} {{ {} }}\n", hmap2.get(&linebuffer).unwrap());
    			}
    		}

    		linebuffer += 1;
    	}

    	if linen1 > linen2 {
    	  	let diff = (linen1 - linen2) + linen2;
			println!("ln {diff} -{{ {} }}", hmap1.get( &diff ).unwrap() );
    	}

    }

    // Showing procs
    if config.proc {
        let procs: Vec<String> = getprocs();
        for strings in procs {
            println!("{strings}");
        }
    }
    // Environment variables
    if config.env {
        for envvars in &archives {
            println!("{envvars} {{ {} }}", env::var(envvars).unwrap().as_str());
        }
    }

    // Date
    if config.date {
    	let systime = SystemTime::now();
    	let diff = systime.duration_since(SystemTime::UNIX_EPOCH);
    	println!("{}", (diff.unwrap().as_secs() as i64).epoch_to_human() );
    }

    // Opening files and showing them
    for names in &archives {
        if !config.env && !config.diff {
            if !config.which {
                let meta = fs::metadata(&names).expect("Error reading file's metadata information.");
            }
            let meta = fs::metadata("/dev/zero").unwrap();

            // " if X = false " is equal to; " if !X "
            // " if X = true " is equal to; " if X "
            if !config.clean {
                if archives.len() > 1 {
                    println!("File Name {{ {names} }}");
                }
                if config.size {
                    println!("Size {{ {} }}", meta.size().size_to_human());
                }

                if config.datetime {
                    println!(
                        "Modified {{ {} }}\nAccessed {{ {} }}\nCreated {{ {} }}",
                        meta.mtime().epoch_to_human(),
                        meta.atime().epoch_to_human(),
                        meta.ctime().epoch_to_human(),
                    );
                }

                if config.lines {
                    println!(
                        "Lines {{ {} }}",
                        fs::read_to_string(names)
                            .expect("Error reading file.")
                            .lines()
                            .count()
                    );
                }

                if config.words {
                    println!(
                        "Words - Letters {{ {:?} }}",
                        fs::read_to_string(names)
                            .expect("Error reading file")
                            .word_count()
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
                            "Owner {{ {} }}",
                            std::str::from_utf8(&ownerout.stdout)
                                .unwrap()
                                .strip_suffix("\n")
                                .unwrap()
                        );
                    } else {
                        println!("Owner {{ Not supported because platform is not detected as Unix. }}");
                    }
                }

                if config.permission {
                    // Permissions method by default will return in bits, if you want the octal chmod
                    // syntax need to use ".mode()".
                    // As Octal is not a type by it self, we need use "format!" macro to convert it in
                    // octal mode, the return is a String.
                    println!(
                        "Permission {{ {:?} }}",
                        format!("{}", meta.permissions().mode()).permission_to_human()
                    );
                }
            }

            // to store information
            // is not storing information yet, because of that we can avoidd "mut" keyword
            let fstring: String;

            if !config.which {
                fstring = String::from_utf8_lossy(&fs::read(names).unwrap()).to_string();
            } else {
                let results: Vec<String> = which( (&names).to_string() );
                for i in results {
                    println!("location {{ {i} }}");
                }
                // Why use process lib and not just "return"? Because this is the proper way to return zero ending the program.
                process::exit(0);
            }


            // Using by reference to not take "len" the ownership of "archives".

            if !config.clean && archives.len() > 1 {
                println!("=================\n");
            }
            if archives.len() > 1 && !config.proc {
                    println!("\ndata {{ {} }}\n------------------------------------------------------", fstring);
            } else {
                if !config.hexa && !config.base64 {
                    println!("\ndata {{ {} }}", fstring);
                }
                if config.hexa {
                    // Hexa mode
                    // Remember; each char will be stored as hexa.
                    let mut buffer: String = String::new();
                    // Split the file's data in lines() and collect each in &str vector.
                    for iteration in fstring.lines().collect::<Vec<&str>>() {
                        // Splits each line in chars
                        for dchar in iteration.chars() {
                            // Transform each char in string and then into bytes data
                            for fchar in dchar.to_string().into_bytes() {
                                // Show each byte char into hexadecimal mode.
                                buffer += &( format!("{:x} ", fchar) ).to_string() ;
                            }
                        }
                    }
                    println!("data {{ {} }}", buffer);
                }

                if config.base64 {
                let file = fs::File::open(names).expect("Error opening file.");
                println!("base64 {{ {} }}", file.encode_base64() );

                }



            }
        }
    }
}
