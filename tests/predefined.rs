mod tar_record;

use basic_tar::{
	BasicTarError, Header, WriteExt,
	raw::{ TypeFlag, BLOCK_LEN }
};
use std::io::Cursor;


/// A test vector to test archive (de-)serialization
struct TestVector {
	archive: &'static[u8],
	expected: Vec<(Header, &'static[u8])>
}
impl TestVector {
	pub fn test_read(self) -> Self {
		// Create reader and iterator
		let mut stream = Cursor::new(self.archive);
		let mut expected = self.expected.iter();
		
		// Read records
		let mut nul_block_counter = 0;
		while nul_block_counter < 2 {
			match tar_record::read_next(&mut stream) {
				Ok((header, payload)) => {
					// Reset nul block counter and get expected value
					nul_block_counter = 0;
					let (_header, _payload) = expected.next().unwrap();
					
					// Verify record
					assert_eq!(&header, _header, "Invalid record {}", _header.path);
					assert_eq!(&payload.as_slice(), _payload, "Invalid record {}", _header.path);
				},
				Err(e) => match e.as_ref().downcast_ref::<BasicTarError>() {
					Some(BasicTarError::EmptyHeader) => nul_block_counter += 1,
					_ => panic!("{}", e)
				}
			}
		}
		self
	}
	pub fn test_write(self) {
		// Write records and EOF blocks
		let mut stream = Cursor::new(Vec::new());
		for (header, payload) in self.expected {
			tar_record::write_next(header, payload, &mut stream).unwrap();
		}
		stream.try_fill(BLOCK_LEN * 2, |_| {}).unwrap();
		
		// Compare data
		let archive = stream.into_inner();
		assert_eq!(archive.len(), self.archive.len());
		assert_eq!(archive.as_slice(), self.archive);
	}
}


#[test]
fn test_read() {
	TestVector {
		archive: include_bytes!("predefined_nul.tar"),
		expected: vec![
			(
				Header {
					path: "predefined_0.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o11, mtime: Some(0o13521071532),
					typeflag: TypeFlag::REGULAR, linkname: None
				},
				include_bytes!("predefined_0.plain")
			),
			
			(
				Header {
					path: "predefined_1.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o12, mtime: Some(0o13521071556),
					typeflag: TypeFlag::REGULAR, linkname: None
				},
				include_bytes!("predefined_1.plain")
			)
		]
	}.test_read().test_write();
	
	TestVector {
		archive: include_bytes!("predefined_bsd.tar"),
		expected: vec![
			(
				Header {
					path: "._predefined_0.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o600, mtime: Some(0o13521657412),
					typeflag: TypeFlag::REGULAR, linkname: None
				},
				include_bytes!("predefined_0.macos")
			),
			(
				Header {
					path: "PaxHeader/predefined_0.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o36, mtime: Some(0o13521657412),
					typeflag: TypeFlag::PAX_SINGLE, linkname: None
				},
				include_bytes!("predefined_0.pax")
			),
			(
				Header {
					path: "predefined_0.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o11, mtime: Some(0o13521657412),
					typeflag: TypeFlag::REGULAR, linkname: None
				},
				include_bytes!("predefined_0.plain")
			),
			
			(
				Header {
					path: "._predefined_1.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o600, mtime: Some(0o13521655376),
					typeflag: TypeFlag::REGULAR, linkname: None
				},
				include_bytes!("predefined_1.macos")
			),
			(
				Header {
					path: "PaxHeader/predefined_1.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o31, mtime: Some(0o13521655376),
					typeflag: TypeFlag::PAX_SINGLE, linkname: None
				},
				include_bytes!("predefined_1.pax")
			),
			(
				Header {
					path: "predefined_1.plain".into(),
					mode: Some(0o644), uid: Some(0o765), gid: Some(0o24),
					size: 0o12, mtime: Some(0o13521655376),
					typeflag: TypeFlag::REGULAR, linkname: None
				},
				include_bytes!("predefined_1.plain")
			)
		]
	}.test_read();
}