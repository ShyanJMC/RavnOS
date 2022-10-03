// Filesystem System lib
use std::fs::{self,File};
// Input Output lib
use std::io::{self,Read};

// Tests to check if libstream works
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search(){
        getprocs();
    }
}

// Input is the string to search
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
            rstr.push( word.to_string() );
        }
    }
    rstr
}

pub fn getprocs() {
        // Read the "/proc" directory.
        let mut entries = fs::read_dir("/proc")
        .unwrap()
        // Takes each part and remap it to "path" method to get full path and then extract only the
        // file name of proc's directory.
        .map(|res| res.map(|e| e.path().file_name()
        .unwrap()
        // Transform each file name to str,
        .to_str()
        .unwrap()
        // Transform each str to string.
        // Take under consideration that as String is stored in heap, to transform
        // any output to string you need first transform to str.
        .to_string() ))
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
        plist.push( i.trim_start_matches('"').trim_end_matches('"').to_string() );
    }
    
    for j in plist {
        // Take each plist's string, split it in chars, go to the first and check if it is a
        // number.
        // If the first char is a number, is appended to strings and then another strings
        // is concatenated to form the final process' status file.
        if j.chars().next().unwrap().is_numeric() {
            buff.push( "/proc/".to_string() + &j.to_string() + &"/status".to_string() );
        }
    }
    
    // Sort the processes vector.
    buff.sort();


    // Call to search a word inside a file.
    for file in buff {
        let vname = file_filter( &file, "Name".to_string() );
        // A simple way to extract strings from the vector, 
        // I know I can use .iter().....etc but with this is more simple and equals quickly.
        for strings in vname {
            println!("{}", strings);
        }
    }

}
