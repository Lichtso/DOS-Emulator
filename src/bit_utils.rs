use std::io;
use std::io::prelude::*;

/*pub unsafe fn transmute_vec<S, T>(mut vec: Vec<S>) -> Vec<T> {
    let ptr = vec.as_mut_ptr() as *mut T;
    let len = vec.len()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    let capacity = vec.capacity()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    std::mem::forget(vec);
    Vec::from_raw_parts(ptr, len, capacity)
}

pub unsafe fn transmute_slice<S, T>(slice: &[S]) -> &[T] {
    let ptr = slice.as_ptr() as *const T;
    let len = slice.len()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    std::slice::from_raw_parts(ptr, len)
}*/

pub unsafe fn transmute_slice_mut<S, T>(slice: &mut [S]) -> &mut [T] {
    let ptr = slice.as_mut_ptr() as *mut T;
    let len = slice.len()*std::mem::size_of::<S>()/std::mem::size_of::<T>();
    std::slice::from_raw_parts_mut(ptr, len)
}

pub fn read_from_buffer_u32(buffer: &[u8]) -> u32 {
    ((buffer[3] as u32)<<24)+((buffer[2] as u32)<<16)+((buffer[1] as u32)<<8)+(buffer[0] as u32)
}

pub fn write_to_buffer_u32(buffer: &mut [u8], value: u32) {
    buffer[3] = (value>>24) as u8;
    buffer[2] = (value>>16) as u8;
    buffer[1] = (value>>8) as u8;
    buffer[0] = value as u8;
}

pub fn read_from_buffer_u16(buffer: &[u8]) -> u16 {
    ((buffer[1] as u16)<<8)+(buffer[0] as u16)
}

pub fn write_to_buffer_u16(buffer: &mut [u8], value: u16) {
    buffer[1] = (value>>8) as u8;
    buffer[0] = value as u8;
}

pub fn write_low_byte_of_u16(dst: &mut u16, src: u8) {
    *dst &= 0xFF00;
    *dst |= src as u16;
}

pub fn write_high_byte_of_u16(dst: &mut u16, src: u8) {
    *dst &= 0x00FF;
    *dst |= (src as u16)<<8;
}

pub fn lsb_mask(length: u32) -> u32 {
    u32::max_value()>>(32-length)
}

pub fn msb_mask(length: u32) -> u32 {
    u32::max_value()<<(32-length)
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
