use std::{io::{self, SeekFrom, BufRead, BufReader, Seek, Error, Read, Write, BufWriter, Cursor}, fs::File};
use byteorder::{BigEndian, LittleEndian, ByteOrder, ReadBytesExt};
use crate::stagedef::{Endianness, StageDef, Game, Vector3, ShortVector3};

#[allow(dead_code)]

const fn from_start(offset: u32) -> Option<SeekFrom> {
    Some(SeekFrom::Start(offset as u64))
}

trait ReadBytesExtSmb {
    fn read_vec3<U: ByteOrder>(&mut self) -> Result<Vector3, Error>;
    fn read_vec3_short<U: ByteOrder>(&mut self) -> Result<ShortVector3, Error>;
    fn read_count_offset<U:ByteOrder> (&mut self) -> Result<CountOffset, Error>; 
}

impl<T: ReadBytesExt> ReadBytesExtSmb for T {
    fn read_vec3<U: ByteOrder>(&mut self) -> Result<Vector3, Error> {
        let x = self.read_f32::<U>()?;
        let y = self.read_f32::<U>()?;
        let z = self.read_f32::<U>()?;
        
        Ok(Vector3 {
            x,
            y,
            z
        })
    }

    fn read_vec3_short<U: ByteOrder>(&mut self) -> Result<ShortVector3, Error> {
        let x = self.read_u16::<U>()?;
        let y = self.read_u16::<U>()?;
        let z = self.read_u16::<U>()?;
        
        Ok(ShortVector3 {
            x,
            y,
            z
        })
    }

    fn read_count_offset<U: ByteOrder>(&mut self) -> Result<CountOffset, Error> {
        let count = self.read_u32::<U>()?;
        let offset = self.read_u32::<U>()?;

        Ok(CountOffset {
            count,
            offset
        })
    }
}

#[derive(Default)]
struct CountOffset {
    count: u32,
    offset: u32,
}

#[derive(Default)]
struct StageDefHeader {
    start_offset: u32,
    fallout_offset: u32,
    goal_list: CountOffset,
}

struct StageDefFormat {
    magic_number_1_offset: Option<SeekFrom>,
    magic_number_2_offset: Option<SeekFrom>,
    collision_header_list_offset: Option<SeekFrom>,
    start_position_ptr_offset: Option<SeekFrom>,
    fallout_position_ptr_offset: Option<SeekFrom>,
    goal_list_offset: Option<SeekFrom>,
    bumper_list_offset: Option<SeekFrom>,
    jamabar_list_offset: Option<SeekFrom>,
    banana_list_offset: Option<SeekFrom>,
    cone_col_list_offset: Option<SeekFrom>,
    sphere_col_list_offset: Option<SeekFrom>,
    cyl_col_list_offset: Option<SeekFrom>,
    fallout_vol_list_offset: Option<SeekFrom>,
    bg_model_list_offset: Option<SeekFrom>,
    fg_model_list_offset: Option<SeekFrom>,
    reflective_model_list_offset: Option<SeekFrom>,
    model_instance_list_offset: Option<SeekFrom>,
    model_ptr_a_list_offset: Option<SeekFrom>,
    model_ptr_b_list_offset: Option<SeekFrom>,
    switch_list_offset: Option<SeekFrom>,
    fog_anim_ptr_offset: Option<SeekFrom>,
    wormhole_list_offset: Option<SeekFrom>,
    fog_ptr_offset: Option<SeekFrom>,
    mystery_3_ptr_offset: Option<SeekFrom>,

}

const SMB2_FORMAT: StageDefFormat = StageDefFormat {
    magic_number_1_offset: from_start(0x0),
    magic_number_2_offset: from_start(0x4),
    collision_header_list_offset: from_start(0x8),
    start_position_ptr_offset: from_start(0x10),
    fallout_position_ptr_offset: from_start(0x14),
    goal_list_offset: from_start(0x18),
    bumper_list_offset: from_start(0x20),
    jamabar_list_offset: from_start(0x28),
    banana_list_offset: from_start(0x30),
    cone_col_list_offset: from_start(0x38),
    sphere_col_list_offset: from_start(0x40),
    cyl_col_list_offset: from_start(0x48),
    fallout_vol_list_offset: from_start(0x50),
    bg_model_list_offset: from_start(0x58),
    fg_model_list_offset: from_start(0x60),
    reflective_model_list_offset: from_start(0x70),
    model_instance_list_offset: from_start(0x84),
    model_ptr_a_list_offset: from_start(0x90),
    model_ptr_b_list_offset: from_start(0x98),
    switch_list_offset: from_start(0xA8),
    fog_anim_ptr_offset: from_start(0xB0),
    wormhole_list_offset: from_start(0xB4),
    fog_ptr_offset: from_start(0xBC),
    mystery_3_ptr_offset: from_start(0xD4),
};

// TODO: Don't store the file in the struct

impl StageDef {
    fn read_stagedef<T: ByteOrder, RW: Read+Write+Seek> (file: RW, game: Game) -> io::Result<StageDef> {
        let mut stagedef = StageDef::default(); 
        let mut reader = BufReader::new(file);
        //let mut removethis_writer = BufWriter::new(file);
        let format = match game {
            //TODO: Implement SMB1 support
            Game::SMB1 => SMB2_FORMAT,
            Game::SMB2 | Game::SMBDX => SMB2_FORMAT,
        };
        
        StageDef::read_header::<T, RW>(&mut stagedef, &mut reader, &format)?;

        Ok(stagedef)
    }

    fn read_header<T: ByteOrder, RW: Read+Write+Seek> (stagedef: &mut StageDef, reader: &mut BufReader<RW>, format: &StageDefFormat) -> io::Result<()> {
        reader.seek(format.magic_number_1_offset.unwrap())?;
        stagedef.magic_number_1 = reader.read_f32::<T>()?; 

        reader.seek(format.magic_number_2_offset.unwrap())?;
        stagedef.magic_number_2 = reader.read_f32::<T>()?; 
/*
        reader.seek(format.start_position_ptr_offset.unwrap())?; 
        let start_position_ptr = from_start(reader.read_u32::<T>()?);
        reader.seek(start_position_ptr.unwrap())?; 
        stagedef.start_position = reader.read_vec3::<T>()?;*/

        Ok(())
    }

    fn read_count_at_offset (count_offset: CountOffset) -> io::Result<()> {
        todo!();
    } 

}

#[cfg(test)]
/// Returns a valid big endian SMB2 main game stagedef with all fields used. 
/// The fields used by the stagedef are as follows:
/// Magic numbers: 0.0, 1,000.0
/// Collision headers: 0 at offset 0 TODO
/// Start position: Offset 0x89c
/// TODO: ...
/// Start position: Vec3: 0.0, 2.75, 14.0, ShortVector3: 0, 0, 0
fn test_smb2_be_stagedef_header() -> Vec<u8> {
    let mut stagedef: Vec<u8> = Vec::new();

    stagedef.append(&mut Vec::from(u32::to_be_bytes(0x00000000)));
    stagedef.append(&mut Vec::from(u32::to_be_bytes(0x447A0000)));
    stagedef.append(&mut Vec::from(u32::to_be_bytes(0x00000000)));
    stagedef.append(&mut Vec::from(u32::to_be_bytes(0x00000000)));


    stagedef
}

#[test]
fn test_stagedef_endianness_test() { 
    let magic_be_test = Vec::from(u32::to_be_bytes(0x447a0000));
    let magic_be_test_bytes = Vec::<u8>::from([0x44, 0x7a, 0x00, 0x00]);
    let magic_le_test = Vec::from(u32::to_le_bytes(0x447a0000));
    let magic_le_test_bytes = Vec::<u8>::from([0x00, 0x00, 0x7a, 0x44]);

    assert_eq!(magic_be_test, magic_be_test_bytes);
    assert_eq!(magic_le_test, magic_le_test_bytes);
}

#[test]
fn test_magic_numbers_big_endian() {
    let file = Cursor::new(test_smb2_be_stagedef_header()); 

    let stagedef = StageDef::read_stagedef::<BigEndian, _>(file, Game::SMB2).unwrap();

    assert_eq!(stagedef.magic_number_1, 0.0);
    assert_eq!(stagedef.magic_number_2, 1000.0);
}


#[test]
fn test_magic_numbers_little_endian() {
    let file = Cursor::new(test_smb2_be_stagedef_header()); 

    let stagedef = StageDef::read_stagedef::<LittleEndian, _>(file, Game::SMB2).unwrap();

    assert_eq!(stagedef.magic_number_1, 0.0);
    assert_eq!(stagedef.magic_number_2, 1000.0);
}
