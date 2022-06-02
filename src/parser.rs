use std::{io::{self, SeekFrom}, fs::File};
use byteorder::{BigEndian, LittleEndian, ByteOrder};
use crate::stagedef::Endianness;

struct CountOffset {
    count: u32,
    offset: u32,
}

struct StageDefParser<'a> {
    file: &'a File,
    endianness: &'a Endianness,
    current_header_position: u32,
    goal_list: CountOffset,
}

trait Serializable {
    fn serialize() -> io::Result<Vec<u8>>;
    fn deserialize() -> io::Result<Box<Self>>;
}

impl Serializable for f32 {
    fn serialize() -> io::Result<Vec<u8>> {
        todo!();
    }

    fn deserialize() -> io::Result<Box<Self>> {
        todo!();
    }
}

impl<'a> StageDefParser<'a> {
    fn read<T: Serializable> () -> io::Result<T> {
       todo!(); 
    }

    fn write<T: Serializable> (value: &T) -> io::Result<()> {
        todo!();
    }

    fn read_count_at_offset<T> (count_offset: CountOffset) -> io::Result<Vec<T>> {
        todo!();
    } 
}
