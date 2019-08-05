//! ## About
//! This crate provides some functionality to read and write __basic/classic oldstyle__ tar archives
//! and some extensions for `io::Read` and `io::Write` to make it easier to work with tar streams.
//!
//! _Note: It is not intended as an high-level allround (un-)packer but as a building block of you
//! want to use the tar format for your own applications – for a high-level solution, take a look
//! at_ [`tar`](https://crates.io/crates/tar)
//!
//! ## How to read a stream
//! To read a tar record from an archive stream, you need to read
//!  1. the header for the next record
//!  2. the payload
//!  3. the padding bytes which pad the payload to a multiple of the block size (512 byte)
//!
//! ### Example:
//! ```
//! # use std::{ convert::TryFrom, error::Error, io::Read };
//! use basic_tar::{
//! 	ReadExt, U64Ext, Header,
//! 	raw::{ self, BLOCK_LEN }
//! };
//!
//! /// Reads the next record from `stream`
//! fn read_next(mut stream: impl Read) -> Result<(Header, Vec<u8>), Box<dyn Error + 'static>> {
//! 	// Read the header
//! 	let mut header_raw = raw::header::raw();
//! 	stream.read_exact(&mut header_raw)?;
//!
//! 	// Parse the header and get the payload lengths
//! 	let header = Header::parse(header_raw)?;
//! 	let payload_len = header.size;
//! 	let payload_total_len = payload_len.ceil_to_multiple_of(BLOCK_LEN as u64);
//!
//! 	// Read the payload
//! 	let mut payload = vec![0; usize::try_from(payload_len)?];
//! 	stream.read_exact(&mut payload)?;
//!
//! 	// Drain the padding and return the record
//! 	let padding_len = usize::try_from(payload_total_len - payload_len)?;
//! 	stream.try_drain(padding_len, |_| {})?;
//! 	Ok((header, payload))
//! }
//! ```
//!
//! ## How to write a stream
//! To write a tar record to an archive archive, you need to write
//!  1. your header
//!  2. your payload
//!  3. the padding bytes to pad your payload to a multiple of the block size (512 byte)
//!
//! ### Example:
//! ```
//! # use std::{ convert::TryFrom, error::Error, io::Write };
//! use basic_tar::{ WriteExt, U64Ext, Header, raw::BLOCK_LEN };
//!
//! /// Writes `header` and `payload` to `stream`
//! fn write_next(header: Header, payload: &[u8], mut stream: impl Write)
//! 	-> Result<(), Box<dyn Error + 'static>>
//! {
//! 	// Serialize the header and write it and the payload
//! 	let header_raw = header.serialize()?;
//! 	stream.write_all(&header_raw)?;
//! 	stream.write_all(payload)?;
//!
//! 	// Write the padding
//! 	let payload_len = payload.len() as u64;
//! 	let padding_len = payload_len.ceil_to_multiple_of(BLOCK_LEN as u64) - payload_len;
//! 	stream.try_fill(usize::try_from(padding_len)?, |_| {})?;
//!
//! 	Ok(())
//! }
//! ```

mod header;
mod helpers;

use std::{
	error::Error,
	fmt::{ self, Display, Formatter }
};
pub use crate::{
	header::{ Header, raw },
	helpers::{ ReadExt, WriteExt, U64Ext }
};


/// A `basic_tar`-related error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum BasicTarError {
	/// An API misuse occurred
	ApiMisuse(&'static str),
	/// The tar header contains invalid data
	InvalidData(&'static str),
	/// The tar header field might be valid but contains an unsupported value
	Unsupported(&'static str),
	/// An empty (all zero) header was found (which is usually part of an end of archive indicator)
	EmptyHeader
}
impl Display for BasicTarError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}
impl Error for BasicTarError {}