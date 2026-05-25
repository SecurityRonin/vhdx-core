#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::{Read, SeekFrom, Seek};
use vhdx::VhdxReader;

fuzz_target!(|data: &[u8]| {
    if let Ok(mut reader) = VhdxReader::from_bytes(data.to_vec()) {
        let size = reader.virtual_disk_size();
        if size > 0 {
            let _ = reader.logical_sector_size();
            let _ = reader.seek(SeekFrom::Start(0));
            let mut buf = [0u8; 512];
            let _ = reader.read(&mut buf);
        }
    }
});
