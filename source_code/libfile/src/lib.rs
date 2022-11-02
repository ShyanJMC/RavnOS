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

/// Trait to work with files' datas and information.
pub trait RavnFile {
	fn size_to_human(&self) -> String;
}

impl RavnFile for u64 {
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
