pub mod raw;

use crate::{
	BasicTarError,
	header::raw::{ StringExt, U64Ext, Checksum }
};


/// A tar header
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Header {
	/// The record's path and name
	pub path: String,
	/// The record's access mode bits (e.g. 0o777)
	pub mode: Option<u64>,
	/// The record's UID
	pub uid: Option<u64>,
	/// The record's GID
	pub gid: Option<u64>,
	/// The record's size
	pub size: u64,
	/// The record's modification time
	pub mtime: Option<u64>,
	/// The record's type
	pub typeflag: u8,
	/// The record's link name
	pub linkname: Option<String>
}
impl Header {
	/// Parses a raw byte block into a classic tar header
	pub fn parse(data: raw::header::Raw) -> Result<Self, BasicTarError> {
		// Check if we have an empty header
		if data.as_ref() == raw::header::raw().as_ref() {
			Err(BasicTarError::EmptyHeader)?
		}
		
		// Parse the basic tar header and verify the checksum
		let tar = raw::header::Header::from(data);
		Checksum::verify(&tar)?;
		
		// Decode the path
		let path = String::from_field(&tar.name)?;
		
		// Decode the mode, UID and GID
		let mode = Option::from_octal_field(&tar.mode)?;
		let uid = Option::from_octal_field(&tar.uid)?;
		let gid = Option::from_octal_field(&tar.gid)?;
		
		// Decode the size and the modification time
		let size = u64::from_octal_field(&tar.size)?;
		let mtime = Option::from_octal_field(&tar.mtime)?;
		
		// Decode link name and create the struct
		let linkname = Option::from_field(&tar.linkname)?;
		Ok(Self{ path, mode, uid, gid, size, typeflag: tar.typeflag[0], mtime, linkname })
	}
	
	/// Serializes the tar header into a raw byte block
	///
	/// _Note: this function can fail because the struct may contain values that cannot be
	/// serialized, e.g. a name longer than 100 bytes or a size greater than 8 GiB_
	pub fn serialize(self) -> Result<raw::header::Raw, BasicTarError> {
		// Create and populate the header
		let mut tar = raw::header::header();
		self.path.into_field(&mut tar.name)?;
		
		self.mode.into_octal_field(&mut tar.mode)?;
		self.uid.into_octal_field(&mut tar.uid)?;
		self.gid.into_octal_field(&mut tar.gid)?;
		
		self.size.into_octal_field(&mut tar.size)?;
		self.mtime.into_octal_field(&mut tar.mtime)?;
		
		tar.typeflag[0] = self.typeflag;
		self.linkname.into_field(&mut tar.linkname)?;
		
		// Write the checksum and return the header
		Checksum::write(&mut tar);
		Ok(tar.into())
	}
}