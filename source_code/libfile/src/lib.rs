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

/// Standard files and read libs
use std::fs::File;
use std::io::Read;
/// Standard environment variable
use std::env::var;
/// Standard path
use std::path::{PathBuf};

extern crate libstream;
use libstream::{Stream};

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
			dreturn = (size/1024).to_string() + "K";
		}
		// Megabyte
		if size > 1048576 && size < 1073741824 {
			dreturn = ((size/1024)/1024).to_string() + "M";
		}
		// Gigabyte
		if size > 1073741824 && size < 1099511627776 {
			dreturn = (((size/1024)/1024)/1024).to_string() + "G";
		}
		// Terabyte
		if size > 1099511627776 {
			dreturn = ((((size/1024)/1024)/2014)/1024).to_string() + "T";
		}
		dreturn
	}
}

impl RavnFile for File {
	/// Detect if file is binary or not
	fn is_binary(&self) -> bool {
		let mut file = self.clone();
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).expect("is_binary function: error to read file");

		for byte in &buffer {
			// If first value is more 127 in size is a binary
			if *byte > 127 {
				return true
			}
		}
		return false
	}


	// Honestly, I needed help with the movement of bits ("<<" ">>") on octets because of that ChatGPT helped me a lot with
	// the "while" loop.
	fn encode_base64(&self) -> String {
		let base64_chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".chars().collect();
		let mut i = 0;
		let mut input = Vec::new();
		let mut output = String::new();

		// Here I use the ownership to use as temporal without the requeriment of keep in memory an unnecessary variable.
		{
			let mut file = self.clone();
			file.read_to_end(&mut input).expect("Error reading file");
		}

		// In this loop ChatGPT helped me a lot with the movements of bits
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

	        let mut triple = (u32::from(octet_a) << 16) | (u32::from(octet_b) << 8) | u32::from(octet_c);

	        if i + 2 >= input.len() {
	            triple <<= 8;
	        }

	        for j in (0..4).rev() {
	            let index = (triple >> (6 * j)) & 0x3F;
	            output.push(base64_chars[index as usize]);
	        }

	        i += 3;
	    }

	    let len = output.len();

		// Remember, in Base64 the "=" and "==" are the end in the string.
		// "=" is called "padding"
	    match len % 4 {
	        0 => (),
	        1 => output.push('='),
	        2 => output.push_str("=="),
	        _ => unreachable!(),
	    }

		// If we not convert in String using UTF-8 as encoding the result will be in u8 only
	    output = match String::from_utf8(output.into_bytes()) {
	        Ok(s) => s,
	        Err(_) => panic!("encode_base64; Error converting to UTF-8"),
	    };

	    output
	}
}


// As which, detects where in PATH is located the binary

pub fn which(binary: String) -> Vec<String>{
	let mut results: Vec<String> = Vec::new();
	// Get the environment variable PATH's value
	let paths = String::from( var("PATH").expect("Error getting PATH environment variable.") );
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
			results.push( ipath.display().to_string() );
		}
	}
	results
}
