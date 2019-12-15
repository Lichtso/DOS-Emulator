use std::io;
use std::io::prelude::*;

pub fn read_from_buffer_u32(buffer: &[u8]) -> u32 {
    ((buffer[3] as u32)<<24)+((buffer[2] as u32)<<16)+((buffer[1] as u32)<<8)+(buffer[0] as u32)
}

pub fn read_from_buffer_u16(buffer: &[u8]) -> u16 {
    ((buffer[1] as u16)<<8)+(buffer[0] as u16)
}

pub fn read_bytes<R>(stream: &mut R, length: u8, buffer: &mut[u8], offset: &mut u8, value: &mut u32) -> io::Result<()> where R: Read {
    stream.read_exact(&mut buffer[*offset as usize..*offset as usize+length as usize])?;
    *value = match length {
        1 => { buffer[*offset as usize] as u32 },
        2 => { read_from_buffer_u16(&buffer[*offset as usize..]) as u32 },
        4 => { read_from_buffer_u32(&buffer[*offset as usize..]) }
        _ => unreachable!()
    };
    *offset += length;
    Ok(())
}
