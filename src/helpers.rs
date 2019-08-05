use std::{
	cmp::min,
	io::{
		self, Read, Write,
		ErrorKind::{ Interrupted, UnexpectedEof, WriteZero }
	}
};


/// An extension for `Read`
pub trait ReadExt {
	/// Tries to fill `buf` completely and calls the position callback `pos_cb` with the amount of
	/// bytes read on *every* successful `read` call
	///
	/// _Note: This function behaves like `read_exact`, except that you will never loose state in
	/// case of an incomplete read - if the error is non-fatal (like `TimedOut`), you can always try
	/// again later if nothing happened_
	fn try_read_exact(&mut self, buf: &mut[u8], pos_cb: impl FnMut(usize))
		-> Result<(), io::Error>;
	
	/// Tries to consume `len` bytes and calls the position callback `pos_cb` with the amount of
	/// bytes drained on *every* successful `read` call
	///
	/// _Note: This function behaves similar to `read_exact` (without buffer), except that you will
	/// never loose state in case of an incomplete write - if the error is non-fatal (like
	/// `TimedOut`), you can always try again later if nothing happened_
	fn try_drain(&mut self, len: usize, pos_cb: impl FnMut(usize)) -> Result<(), io::Error>;
}
impl<T: Read> ReadExt for T {
	fn try_read_exact(&mut self, mut buf: &mut[u8], mut pos_cb: impl FnMut(usize))
		-> Result<(), io::Error>
	{
		'read_loop: while !buf.is_empty() {
			match self.read(&mut buf) {
				Err(ref e) if e.kind() == Interrupted => continue 'read_loop,
				Err(e) => Err(e)?,
				Ok(0) => Err(io::Error::from(UnexpectedEof))?,
				Ok(len) => {
					buf = &mut buf[len..];
					pos_cb(len)
				}
			}
		}
		Ok(())
	}
	fn try_drain(&mut self, mut len: usize, mut pos_cb: impl FnMut(usize))
		-> Result<(), io::Error>
	{
		// Read len bytes
		while len > 0 {
			// Create buffer and fill it
			let buf = &mut[0; 4096][.. min(len, 4096)];
			self.try_read_exact(buf, |read| {
				len -= read;
				pos_cb(read)
			})?
		}
		Ok(())
	}
}


/// An extension for `Write`
pub trait WriteExt {
	/// Tries to write `data` completely and calls the position callback `pos_cb` with the amount of
	/// bytes written on *every* successful `write` call
	///
	/// _Note: This function behaves like `write_exact`, except that you will never loose state in
	/// case of an incomplete write - if the error is non-fatal (like `TimedOut`), you can always
	/// try again later from the last position as if nothing happened_
	fn try_write_exact(&mut self, data: &[u8], counter: impl FnMut(usize))
		-> Result<(), io::Error>;
	
	/// Tries to write `len` zero bytes and calls the position callback `pos_cb` with the amount of
	/// bytes written on *every* successful `write` call
	///
	/// _Note: This function behaves similar to `write_exact` (without data), except that you will
	/// never loose state in case of an incomplete write - if the error is non-fatal (like
	/// `TimedOut`), you can always try again later if nothing happened_
	fn try_fill(&mut self, len: usize, counter: impl FnMut(usize)) -> Result<(), io::Error>;
}
impl<T: Write> WriteExt for T {
	fn try_write_exact(&mut self, mut data: &[u8], mut pos_cb: impl FnMut(usize))
		-> Result<(), io::Error>
	{
		'write_loop: while !data.is_empty() {
			match self.write(&data) {
				Err(ref e) if e.kind() == Interrupted => continue 'write_loop,
				Err(e) => Err(e)?,
				Ok(0) => Err(io::Error::from(WriteZero))?,
				Ok(len) => {
					data = &data[len..];
					pos_cb(len);
				}
			}
		}
		Ok(())
	}
	fn try_fill(&mut self, mut len: usize, mut pos_cb: impl FnMut(usize))
		-> Result<(), io::Error>
	{
		// Write len zero bytes
		while len > 0 {
			// Create buffer and fill it
			let buf = &mut[0; 4096][.. min(len, 4096)];
			self.try_write_exact(buf, |written| {
				len -= written;
				pos_cb(written)
			})?
		}
		Ok(())
	}
}


/// An extension for `u64`
pub trait U64Ext {
	/// Ceils `self` to an integer multiple of `num`
	fn ceil_to_multiple_of(&self, num: Self) -> Self;
}
impl U64Ext for u64 {
	fn ceil_to_multiple_of(&self, num: Self) -> Self {
		match *self % num {
			0 => *self,
			r => *self + (num - r)
		}
	}
}