[![docs.rs](https://docs.rs/basic_tar/badge.svg)](https://docs.rs/basic_tar)
[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/basic_tar.svg)](https://crates.io/crates/basic_tar)
[![Download numbers](https://img.shields.io/crates/d/basic_tar.svg)](https://crates.io/crates/basic_tar)
[![Travis CI](https://travis-ci.org/KizzyCode/basic_tar.svg?branch=master)](https://travis-ci.org/KizzyCode/basic_tar)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/basic_tar?svg=true)](https://ci.appveyor.com/project/KizzyCode/basic-tar)
[![dependency status](https://deps.rs/crate/basic_tar/0.1.3/status.svg)](https://deps.rs/crate/basic_tar/0.1.3)

# basic_tar
Welcome to `basic_tar` 🎉


## About
This crate provides some functionality to read and write __basic/classic oldstyle__ tar archives and
some extensions for `io::Read` and `io::Write` to make it easier to work with tar streams.

_Note: It is not intended as an high-level allround (un-)packer but as a building block of you want
to use the tar format for your own applications – for a high-level solution, take a look at_
[`tar`](https://crates.io/crates/tar)


## How to read a stream
To read a tar record from an archive stream, you need to read
 1. the header for the next record
 2. the payload
 3. the padding bytes which pad the payload to a multiple of the block size (512 byte)

### Example:
```rust
use std::{ convert::TryFrom, error::Error, io::Read };
use basic_tar::{
	ReadExt, U64Ext, Header,
	raw::{ self, BLOCK_LEN }
};

/// Reads the next record from `stream`
fn read_next(mut stream: impl Read) -> Result<(Header, Vec<u8>), Box<dyn Error + 'static>> {
	// Read the header
	let mut header_raw = raw::header::raw();
	stream.read_exact(&mut header_raw)?;

	// Parse the header and get the payload lengths
	let header = Header::parse(header_raw)?;
	let payload_len = header.size;
	let payload_total_len = payload_len.ceil_to_multiple_of(BLOCK_LEN as u64);

	// Read the payload
	let mut payload = vec![0; usize::try_from(payload_len)?];
	stream.read_exact(&mut payload)?;

	// Drain the padding and return the record
	let padding_len = usize::try_from(payload_total_len - payload_len)?;
	stream.try_drain(padding_len, |_| {})?;
	Ok((header, payload))
}
```


## How to write a stream
To write a tar record to an archive stream, you need to write
 1. your header
 2. your payload
 3. the padding bytes to pad your payload to a multiple of the block size (512 byte)

### Example:
```rust
use std::{ convert::TryFrom, error::Error, io::Write };
use basic_tar::{ WriteExt, U64Ext, Header, raw::BLOCK_LEN };

/// Writes `header` and `payload` to `stream`
fn write_next(header: Header, payload: &[u8], mut stream: impl Write)
	-> Result<(), Box<dyn Error + 'static>>
{
	// Serialize the header and write it and the payload
	let header_raw = header.serialize()?;
	stream.write_all(&header_raw)?;
	stream.write_all(payload)?;

	// Write the padding
	let payload_len = payload.len() as u64;
	let padding_len = payload_len.ceil_to_multiple_of(BLOCK_LEN as u64) - payload_len;
	stream.try_fill(usize::try_from(padding_len)?, |_| {})?;

	Ok(())
}
```