//! The raw representation of the TAR header fields and some byte constants

use crate::BasicTarError;
use std::{ iter, mem };


/// The length of a tar block
pub const BLOCK_LEN: usize = 512;


/// Defines the classic old-style tar header
pub mod header {
	use super::{ mem, BLOCK_LEN };
	
	/// A raw header block
	pub type Raw = [u8; BLOCK_LEN];
	/// Creates a new all-zero raw header
	pub const fn raw() -> Raw {
		[0; BLOCK_LEN]
	}
	
	/// The 1:1-byte representation of the classic old-style tar header
	#[repr(packed)]
	#[derive(Copy, Clone)]
	pub struct Header {
		pub name: [u8; 100],
		pub mode: [u8; 8],
		pub uid: [u8; 8],
		pub gid: [u8; 8],
		pub size: [u8; 12],
		pub mtime: [u8; 12],
		pub checksum: [u8; 8],
		pub typeflag: [u8; 1],
		pub linkname: [u8; 100],
		pub _extra: [u8; 243],
		pub _pad: [u8; 12]
	}
	/// Creates a new all-zero header
	pub fn header() -> Header {
		Header::from(raw())
	}
	impl From<Raw> for Header {
		fn from(raw: Raw) -> Self {
			assert_eq!(mem::size_of_val(&raw), mem::size_of::<Self>());
			unsafe{ mem::transmute(raw) }
		}
	}
	impl Into<Raw> for Header {
		fn into(self) -> Raw {
			assert_eq!(mem::size_of_val(&self), mem::size_of::<Raw>());
			unsafe{ mem::transmute(self) }
		}
	}
}


/// The type flags which indicate the record type
pub struct TypeFlag;
impl TypeFlag {
	/// The type flag for a regular file
	pub const REGULAR: u8 = b'0';
	/// The type flag for a hardlink
	pub const HARDLINK: u8 = b'1';
	/// The type flag for a symlink
	pub const SYMLINK: u8 = b'2';
	/// The type flag for a character device
	pub const CHAR_DEV: u8 = b'3';
	/// The type flag for a block device
	pub const BLOCK_DEV: u8 = b'4';
	/// The type flag for a directory
	pub const DIRECTORY: u8 = b'5';
	/// The type flag for a FIFO-node (named pipe)
	pub const FIFO_NODE: u8 = b'6';
	/// Reserved to represent a file to which an implementation has associated some high-performance
	/// attribute
	#[doc(hidden)] pub const RESERVED: u8 = b'7';
	/// The type flag for a pax interchange record that only affects the next file
	pub const PAX_SINGLE: u8 = b'x';
	/// The type flag for a pax interchange record that affects all subsequent files
	pub const PAX_GLOBAL: u8 = b'g';
}


/// Helpers for checksum verification
pub(in crate::header) struct Checksum;
impl Checksum {
	/// Computes the checksum over `raw` and writes it to the raw header
	pub fn write(tar: &mut header::Header) {
		Self::compute(*tar).into_octal_field(&mut tar.checksum)
			.expect("We should always be able to octal-encode an `u32` into 8 bytes");
	}
	/// Verifies the checksum for `raw`
	pub fn verify(tar: &header::Header) -> Result<(), BasicTarError> {
		match Self::compute(*tar) == u64::from_octal_field(&tar.checksum)? {
			true => Ok(()),
			false => Err(BasicTarError::InvalidData("Invalid header checksum"))
		}
	}
	
	/// Computes the checksum
	fn compute(tar: header::Header) -> u64 {
		let raw: header::Raw = tar.into();
		raw[..148].iter().chain([b' '; 8].iter()).chain(raw[156..].iter())
			.fold(0, |sum, byte| sum + (*byte as u64))
	}
}


/// An extension for `u64`
pub(in crate::header) trait U64Ext: Sized {
	/// Creates a new `u64` from an octal string
	fn from_octal_field(field: &[u8]) -> Result<Self, BasicTarError>;
	/// Creates an octal string from `self`
	fn into_octal_field(self, field: &mut[u8]) -> Result<(), BasicTarError>;
}
impl U64Ext for Option<u64> {
	fn from_octal_field(field: &[u8]) -> Result<Self, BasicTarError> {
		let string = Option::<String>::from_terminated_field(field)?;
		let octal = match string.as_ref().map(|s| s.trim_end()) {
			Some(octal) if octal.len() > 0 => octal,
			_ => return Ok(None)
		};
		
		let num = u64::from_str_radix(&octal, 8)
			.map_err(|_| BasicTarError::InvalidData("Invalid octal number"))?;
		Ok(Some(num))
	}
	fn into_octal_field(self, field: &mut[u8]) -> Result<(), BasicTarError> {
		// Serialize the value
		let num = self.map(|num| format!("{:o}", num)).unwrap_or_default();
		
		// Compute the amount of "0"-literals to prepend
		let available = field.len().checked_sub(1).unwrap_or(0);
		let pad = available.checked_sub(num.len()).unwrap_or(0);
		
		// Create the padded string and write it to the field
		let num: String = iter::repeat('0').take(pad).chain(num.chars()).collect();
		num.into_terminated_field(field)
	}
}
impl U64Ext for u64 {
	fn from_octal_field(field: &[u8]) -> Result<Self, BasicTarError> {
		Option::from_octal_field(field)?
			.ok_or(BasicTarError::InvalidData("Required field is empty"))
	}
	fn into_octal_field(self, field: &mut[u8]) -> Result<(), BasicTarError> {
		Some(self).into_octal_field(field)
	}
}


/// An extension for `String`
pub(in crate::header) trait StringExt: Sized {
	/// Creates a new string from a (potentially NUL-terminated) tar field
	fn from_field(field: &[u8]) -> Result<Self, BasicTarError>;
	/// Writes `self` to `field` and NUL-pads the string is field is longer than the value
	fn into_field(self, field: &mut[u8]) -> Result<(), BasicTarError>;
	
	/// Creates a new string from an always space-terminated tar field (after NUL-trimming) and
	/// removes the space
	fn from_terminated_field(field: &[u8]) -> Result<Self, BasicTarError>;
	/// Writes `self` to `field` and ensures that at least the last byte in the field is space
	fn into_terminated_field(self, field: &mut[u8]) -> Result<(), BasicTarError>;
}
impl StringExt for Option<String> {
	fn from_field(field: &[u8]) -> Result<Self, BasicTarError> {
		// Trim data
		let nul = field.iter().position(|b| *b == 0x00).unwrap_or(field.len());
		let data = field[..nul].to_vec();
		
		// Parse string
		match data.is_empty() {
			true => Ok(None),
			false => {
				let string = String::from_utf8(data.to_vec())
					.map_err(|_| BasicTarError::Unsupported("Header field is not UTF-8"))?;
				Ok(Some(string))
			}
		}
	}
	fn into_field(self, field: &mut[u8]) -> Result<(), BasicTarError> {
		// Check if we can write the field
		if field.len() < self.as_ref().map(|string| string.len()).unwrap_or(0) {
			Err(BasicTarError::ApiMisuse("`field` is too small to hold the value"))?
		}
		let string = self.as_ref().map(|string| string.as_str()).unwrap_or_default();
		
		// NUL-terminate the string and copy it to the field
		let nul_terminated = string.bytes().chain(iter::repeat(0));
		field.iter_mut().zip(nul_terminated).for_each(|(field, byte)| *field = byte);
		Ok(())
	}
	fn from_terminated_field(field: &[u8]) -> Result<Self, BasicTarError> {
		let space = field.iter().position(|b| *b == b' ').unwrap_or(field.len());
		Self::from_field(&field[..space])
	}
	fn into_terminated_field(self, field: &mut[u8]) -> Result<(), BasicTarError> {
		// Get the index of the last byte
		let last = field.len().checked_sub(1)
			.ok_or(BasicTarError::ApiMisuse("`field` is too small to hold the value"))?;
		
		// Write the value and set the last byte
		self.into_field(&mut field[..last])?;
		field[last] = 0;
		Ok(())
	}
}
impl StringExt for String {
	fn from_field(field: &[u8]) -> Result<Self, BasicTarError> {
		Option::<String>::from_field(field)?
			.ok_or(BasicTarError::InvalidData("Required field is empty"))
	}
	fn into_field(self, field: &mut[u8]) -> Result<(), BasicTarError> {
		Some(self).into_field(field)
	}
	fn from_terminated_field(field: &[u8]) -> Result<Self, BasicTarError> {
		Option::<String>::from_terminated_field(field)?
			.ok_or(BasicTarError::InvalidData("Required field is empty"))
	}
	fn into_terminated_field(self, field: &mut[u8]) -> Result<(), BasicTarError> {
		Some(self).into_terminated_field(field)
	}
}
