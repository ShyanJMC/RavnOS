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
//! Copyright; Joaquin "ShyanJMC" Crespo - 2022

/// Standard environment variable
use std::env::var;
/// Standard files and read libs
use std::fs::File;
use std::io::Read;
/// Standard path
use std::path::PathBuf;

extern crate libstream;
use libstream::Stream;

/// Trait to work with files' datas and information.
pub trait RavnSizeFile {
    fn size_to_human(&self) -> String;
}

pub trait RavnFile {
    fn is_binary(&self) -> bool;
    fn encode_base64(&self) -> String;
}

impl RavnSizeFile for u64 {
    /// takes as self input (bytes) the size of file/directory and returns String with human size.
    fn size_to_human(&self) -> String {
        let size = self.clone();
        let mut dreturn = String::new();
        // Bytes
        if size <= 1024 {
            dreturn = size.to_string() + "B";
        }
        // Kilobyte
        if size >= 1024 && size < 1048576 {
            dreturn = (size / 1024).to_string() + "K";
        }
        // Megabyte
        if size > 1048576 && size < 1073741824 {
            dreturn = ((size / 1024) / 1024).to_string() + "M";
        }
        // Gigabyte
        if size > 1073741824 && size < 1099511627776 {
            dreturn = (((size / 1024) / 1024) / 1024).to_string() + "G";
        }
        // Terabyte
        if size > 1099511627776 {
            dreturn = ((((size / 1024) / 1024) / 2014) / 1024).to_string() + "T";
        }
        dreturn
    }
}

impl RavnFile for File {
    /// Detect if file is binary or not
    fn is_binary(&self) -> bool {
        let mut file = self.clone();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("is_binary function: error to read file");

        for byte in &buffer {
            // If first value is more 127 in size is a binary
            if *byte > 127 {
                return true;
            }
        }
        return false;
    }

    // Honestly, I needed help with the movement of bits ("<<" ">>") on octets because of that ChatGPT helped me a lot with
    // the "while" loop.
    fn encode_base64(&self) -> String {
        let base64_chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            .chars()
            .collect();
        let mut i = 0;
        let mut input = Vec::new();
        let mut output = String::new();

        {
            let mut file = self.clone();
            match file.read_to_end(&mut input) {
                Ok(_) => (),
                Err(e) => return format!("Error reading file: {}", e),
            };
        }

        while i < input.len() {
            let mut octet_a = 0u8;
            let mut octet_b = 0u8;
            let mut octet_c = 0u8;

            octet_a = input[i];

            if i + 1 < input.len() {
                octet_b = input[i + 1];
            }

            if i + 2 < input.len() {
                octet_c = input[i + 2];
            }

            let triple = (u32::from(octet_a) << 16) | (u32::from(octet_b) << 8) | u32::from(octet_c);

            for j in (0..4).rev() {
                let index = (triple >> (6 * j)) & 0x3F;
                output.push(base64_chars[index as usize]);
            }

            i += 3;
        }

        let len = output.len();

        match len % 4 {
            0 => (),
            1 => output.push('='),
            2 => output.push_str("=="),
            _ => unreachable!(),
        }

        output
    }

}

// As which, detects where in PATH is located the binary

pub fn which(binary: String) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    // Get the environment variable PATH's value
    let paths = String::from(var("PATH").expect("Error getting PATH environment variable."));
    // Split each path and save it into a vector of str, because the return is provided by split we know the size when is passed to collect because
    // of that we can use str instead of String.
    let inv_paths: Vec<&str> = paths.split(':').collect();
    // Read each directory
    let mut entries: Vec<PathBuf> = Vec::new();
    for ivalue in inv_paths {
        for i in ivalue.to_string().readdir() {
            entries.push(i.clone());
        }
    }
    for ipath in entries {
        if ipath.display().to_string().contains(&binary) {
            results.push(ipath.display().to_string());
        }
    }
    results
}


// Here ChatGPT helped me again with bits movement
// Vec<u8> are the bits, and String if the error if be
pub fn decode_base64(base64: &String) -> Result<Vec<u8>, String> {
    const BASE64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                   abcdefghijklmnopqrstuvwxyz\
                                       0123456789+/";

    let error_str: &str = "Error or invalid base64 character";
    let endata: &str = "unexpected end of data";
    let input = base64.clone();
    let mut output = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c1) = chars.next() {
        if c1 == '=' {
            break; // padding character, end of data
        }
        let i1 = BASE64_ALPHABET.iter().position(|&x| x == c1 as u8).ok_or_else(|| error_str.to_string()).unwrap();

        let c2 = chars.next().ok_or(&endata).unwrap();
        if c2 == '=' {
            break; // padding character, end of data
        }
        let i2 = BASE64_ALPHABET.iter().position(|&x| x == c2 as u8).ok_or_else(|| error_str.to_string()).unwrap();

        let c3 = chars.next().ok_or(&endata).unwrap();
        let i3 = if c3 == '=' {
            0 // padding character, equivalent to zero bits
        } else {
            BASE64_ALPHABET.iter().position(|&x| x == c3 as u8).ok_or(&error_str).unwrap()
        };

        let c4 = chars.next().ok_or(&endata).unwrap();
        let i4 = if c4 == '=' {
            0 // padding character, equivalent to zero bits
        } else {
            BASE64_ALPHABET.iter().position(|&x| x == c4 as u8).ok_or(&error_str).unwrap()
        };

        let byte1 = (i1 << 2) | (i2 >> 4);
        let byte2 = ((i2 & 0x0f) << 4) | (i3 >> 2);
        let byte3 = ((i3 & 0x03) << 6) | i4;
        output.push(byte1);
        if c3 != '=' {
            output.push(byte2);
        }
        if c4 != '=' {
            output.push(byte3);
        }
    }

    // Convertir el vector de usize a u8
    let output_u8: Vec<u8> = output.iter().map(|&x| x as u8).collect();

    // Devolver el resultado como u8
    Ok(output_u8)
}
