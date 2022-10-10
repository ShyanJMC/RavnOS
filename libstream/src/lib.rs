// Filesystem System lib
use std::fs::{self, File};
// Input Output lib
use std::io::{self, Read};

// Tests to check if libstream works
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search() {
        getprocs();
    }
    #[test]
    fn permissionchmod() {
        permission_to_human(&10644);
    }
}

// Unix time to localtime
//pub fn

// Outputs
pub trait OutputMode {
    fn permission_to_human(&self) -> Vec<&'static str>;
}

impl OutputMode for String {
    // Translate octal permission input to human redeable.
    fn permission_to_human(&self) -> Vec<&'static str> {
        // Positions; 0 setuid/setguid/stickybit, 1 owner, 2 group, 3 others.
        // // Positions of string; 0-1 setguid/setguid/stickybit, 2 owner, 3 group, 4 others
        let mut vecper: Vec<&str> = Vec::new();
        let buffer: Vec<u32> = self
            .clone()
            .chars()
            .map(|d| d.to_digit(10).unwrap())
            .collect();
        let mut fbuffer: Vec<u32> = Vec::new();
        for word in &buffer {
            fbuffer.push(*word);
        }

        // Setuid / Setguid / Stickybit
        match fbuffer[0] {
            0 => vecper.push(" "),
            1 => vecper.push("sticky bit"),
            2 => vecper.push("seted user id"),
            4 => vecper.push("seted group id"),
            _ => vecper.push("error"),
        }

        // Quick fix, because sometimes the octal output (self) is 5 digit and sometimes is 6
        // digit lenght.
        if fbuffer.len() == 5 {
            // Owner
            match fbuffer[2] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Group
            match fbuffer[3] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Others
            match fbuffer[4] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }
        }

        if fbuffer.len() == 6 {
            // Owner
            match fbuffer[3] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Group
            match fbuffer[4] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }

            // Others
            match fbuffer[5] {
                0 => vecper.push("---"),
                1 => vecper.push("--x"),
                2 => vecper.push("-w-"),
                3 => vecper.push("-wx"),
                4 => vecper.push("r--"),
                5 => vecper.push("r-x"),
                6 => vecper.push("rw-"),
                7 => vecper.push("rwx"),
                _ => vecper.push("error"),
            }
        }

        vecper
    }
}

// Filename is the file's name to open.
// Input is the string to search
// Search for "input" into the file and returns strings.
pub fn file_filter(filename: &String, input: String) -> Vec<String> {
    // Read the file as string and then save to lines buffer.
    let mut buffer1 = String::new();

    // Open file
    let mut file = File::open(filename).unwrap();

    // Read file to string and save in buffer1
    file.read_to_string(&mut buffer1).unwrap();

    // Split in lines
    let lbuffer = buffer1.lines();

    // If you return directly from "for" loop will be taken as '()'
    // instead String.
    let mut rstr: Vec<String> = Vec::new();

    for word in lbuffer {
        if word.contains(&input) {
            rstr.push(word.to_string());
        }
    }
    rstr
}

// Unix.
// Get processes and information.
pub fn getprocs() {
    // Read the "/proc" directory.
    let mut entries = fs::read_dir("/proc")
        .unwrap()
        // Takes each part and remap it to "path" method to get full path and then extract only the
        // file name of proc's directory.
        .map(|res| {
            res.map(|e| {
                e.path()
                    .file_name()
                    .unwrap()
                    // Transform each file name to str,
                    .to_str()
                    .unwrap()
                    // Transform each str to string.
                    // Take under consideration that as String is stored in heap, to transform
                    // any output to string you need first transform to str.
                    .to_string()
            })
        })
        // Collect each map's returns into a Vec<String>
        .collect::<Result<Vec<String>, io::Error>>()
        // As collect returns a Result this is a must.
        .unwrap();

    // Sort the strings.
    entries.sort();

    // To variables for; procs lists and buffer.
    let mut plist: Vec<String> = Vec::new();
    let mut buff: Vec<String> = Vec::new();
    //let mut fbuff;

    for i in entries {
        // Clean the double quotes ("") at variable's start and end and then pass to String.
        // Then with the file name cleaned, push it to plist vector variable.
        plist.push(i.trim_start_matches('"').trim_end_matches('"').to_string());
    }

    for j in plist {
        // Take each plist's string, split it in chars, go to the first and check if it is a
        // number.
        // If the first char is a number, is appended to strings and then another strings
        // is concatenated to form the final process' status file.
        if j.chars().next().unwrap().is_numeric() {
            buff.push("/proc/".to_string() + &j.to_string() + &"/status".to_string());
        }
    }

    // Sort the processes vector.
    buff.sort();

    // Call to search a word inside a file.
    for file in buff {
        let vname = file_filter(&file, "Name".to_string());
        let vpid = file_filter(&file, "Pid".to_string());
        let vppid = file_filter(&file, "PPid".to_string());
        let vuid = file_filter(&file, "Uid".to_string());
        // A simple way to extract strings from the vector,
        // I know I can use .iter().....etc but with this is more simple and equals quickly.
        for strings in &vname {
            for pid in &vpid {
                for ppid in &vppid {
                    for owner in &vuid {
                        println!("{}\t{}\t{}\t{}", strings, pid, ppid, owner);
                    }
                }
            }
        }
    }
}
