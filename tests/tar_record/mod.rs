use std::{
	convert::TryFrom, error::Error,
	io::{ Read, Write }
};
use basic_tar::{
	ReadExt, WriteExt, U64Ext, Header,
	raw::{ self, BLOCK_LEN }
};


/// Reads the next record from `stream`
pub fn read_next(mut stream: impl Read) -> Result<(Header, Vec<u8>), Box<dyn Error + 'static>> {
	// Read the header using `try_read_exact` - useful to resume later in case of an error
	let mut header_raw = raw::header::raw();
	stream.read_exact(&mut header_raw)?;
	
	// Parse the header and get the payload lengths
	let header = Header::parse(header_raw)?;
	let payload_len = header.size;
	let payload_total_len = payload_len.ceil_to_multiple_of(BLOCK_LEN as u64);
	
	// Read the payload using `try_read_exact` - useful to resume later in case of an error
	let mut payload = vec![0; usize::try_from(payload_len)?];
	stream.read_exact(&mut payload)?;
	
	// Drain the padding and return the record
	let padding_len = usize::try_from(payload_total_len - payload_len)?;
	stream.try_drain(padding_len, |_| {})?;
	Ok((header, payload))
}


/// Writes `header` and `payload` to `stream`
pub fn write_next(header: Header, payload: &[u8], mut stream: impl Write)
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