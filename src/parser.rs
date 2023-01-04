//! Handles parsing of an uncompressed Monkey Ball stage binary.
use crate::stagedef::{
    Endianness, Game, GlobalStagedefObject, Goal, GoalType, ShortVector3, StageDef, Vector3,
};
use anyhow::Result;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

const fn from_start(offset: u32) -> SeekFrom {
    SeekFrom::Start(offset as u64)
}

const fn from_relative(start: u32, offset: u32) -> SeekFrom {
    SeekFrom::Start(start as u64 + offset as u64)
}

trait ReadBytesExtSmb {
    fn read_vec3<U: ByteOrder>(&mut self) -> Result<Vector3>;
    fn read_vec3_short<U: ByteOrder>(&mut self) -> Result<ShortVector3>;
    fn read_offset<U: ByteOrder>(&mut self) -> Result<FileOffset>;
    fn read_count_offset<U: ByteOrder>(&mut self) -> Result<FileOffset>;
    fn read_goal<U: ByteOrder>(&mut self) -> Result<Goal>;
}

impl<T: ReadBytesExt> ReadBytesExtSmb for T {
    fn read_vec3<U: ByteOrder>(&mut self) -> Result<Vector3> {
        let x = self.read_f32::<U>()?;
        let y = self.read_f32::<U>()?;
        let z = self.read_f32::<U>()?;

        Ok(Vector3 { x, y, z })
    }

    fn read_vec3_short<U: ByteOrder>(&mut self) -> Result<ShortVector3> {
        let x = self.read_u16::<U>()?;
        let y = self.read_u16::<U>()?;
        let z = self.read_u16::<U>()?;

        Ok(ShortVector3 { x, y, z })
    }

    fn read_offset<U: ByteOrder>(&mut self) -> Result<FileOffset> {
        let offset = from_start(self.read_u32::<U>()?);

        Ok(FileOffset::OffsetOnly(offset))
    }

    fn read_count_offset<U: ByteOrder>(&mut self) -> Result<FileOffset> {
        let count = self.read_u32::<U>()?;
        let offset = from_start(self.read_u32::<U>()?);

        Ok(FileOffset::CountOffset(count, offset))
    }

    fn read_goal<U: ByteOrder>(&mut self) -> Result<Goal> {
        let position = self.read_vec3::<U>()?;
        let rotation = self.read_vec3_short::<U>()?;

        let goal_type: GoalType = FromPrimitive::from_u8(self.read_u8()?).unwrap_or_default();
        self.read_u8()?;

        Ok(Goal {
            position,
            rotation,
            goal_type,
        })
    }
}

trait SeekExtSmb {
    fn seek_fileoffset(&mut self, offset: FileOffset) -> io::Result<u64>;
}

impl<T: Seek> SeekExtSmb for T {
    fn seek_fileoffset(&mut self, offset: FileOffset) -> io::Result<u64> {
        match offset {
            FileOffset::Unused => Err(io::Error::new(io::ErrorKind::Other, "Attempted to seek to an unused value")),
            FileOffset::OffsetOnly(o) => self.seek(o),
            FileOffset::CountOffset(_, o) => self.seek(o),
        }
    }
}

#[derive(Default)]
enum FileOffset {
    #[default]
    Unused,
    OffsetOnly(SeekFrom),
    CountOffset(u32, SeekFrom),
}

/// A struct that defines the file header format for a Monkey Ball stagedef file. The fields define
/// the location from the start of the file in which the given structure can be found. These fields
/// are optional, for situations where certain structures are not in a particular game (for
/// example, Super Monkey Ball 1 does not have wormholes).
#[derive(Default)]
struct StageDefFileHeaderFormat {
    magic_number_1_offset: FileOffset,
    magic_number_2_offset: FileOffset,
    collision_header_list_offset: FileOffset,
    start_position_ptr_offset: FileOffset,
    fallout_position_ptr_offset: FileOffset,
    goal_list_offset: FileOffset,
    bumper_list_offset: FileOffset,
    jamabar_list_offset: FileOffset,
    banana_list_offset: FileOffset,
    cone_col_list_offset: FileOffset,
    sphere_col_list_offset: FileOffset,
    cyl_col_list_offset: FileOffset,
    fallout_vol_list_offset: FileOffset,
    bg_model_list_offset: FileOffset,
    fg_model_list_offset: FileOffset,
    reflective_model_list_offset: FileOffset,
    model_instance_list_offset: FileOffset,
    model_ptr_a_list_offset: FileOffset,
    model_ptr_b_list_offset: FileOffset,
    switch_list_offset: FileOffset,
    fog_anim_ptr_offset: FileOffset,
    wormhole_list_offset: FileOffset,
    fog_ptr_offset: FileOffset,
    mystery_3_ptr_offset: FileOffset,
}

const SMB2_FILE_HEADER_FORMAT: StageDefFileHeaderFormat = StageDefFileHeaderFormat {
    magic_number_1_offset: FileOffset::OffsetOnly(from_start(0x0)),
    magic_number_2_offset: FileOffset::OffsetOnly(from_start(0x4)),
    collision_header_list_offset: FileOffset::OffsetOnly(from_start(0x8)),
    start_position_ptr_offset: FileOffset::OffsetOnly(from_start(0x10)),
    fallout_position_ptr_offset: FileOffset::OffsetOnly(from_start(0x14)),
    goal_list_offset: FileOffset::OffsetOnly(from_start(0x18)),
    bumper_list_offset: FileOffset::OffsetOnly(from_start(0x20)),
    jamabar_list_offset: FileOffset::OffsetOnly(from_start(0x28)),
    banana_list_offset: FileOffset::OffsetOnly(from_start(0x30)),
    cone_col_list_offset: FileOffset::OffsetOnly(from_start(0x38)),
    sphere_col_list_offset: FileOffset::OffsetOnly(from_start(0x40)),
    cyl_col_list_offset: FileOffset::OffsetOnly(from_start(0x48)),
    fallout_vol_list_offset: FileOffset::OffsetOnly(from_start(0x50)),
    bg_model_list_offset: FileOffset::OffsetOnly(from_start(0x58)),
    fg_model_list_offset: FileOffset::OffsetOnly(from_start(0x60)),
    reflective_model_list_offset: FileOffset::OffsetOnly(from_start(0x70)),
    model_instance_list_offset: FileOffset::OffsetOnly(from_start(0x84)),
    model_ptr_a_list_offset: FileOffset::OffsetOnly(from_start(0x90)),
    model_ptr_b_list_offset: FileOffset::OffsetOnly(from_start(0x98)),
    switch_list_offset: FileOffset::OffsetOnly(from_start(0xA8)),
    fog_anim_ptr_offset: FileOffset::OffsetOnly(from_start(0xB0)),
    wormhole_list_offset: FileOffset::OffsetOnly(from_start(0xB4)),
    fog_ptr_offset: FileOffset::OffsetOnly(from_start(0xBC)),
    mystery_3_ptr_offset: FileOffset::OffsetOnly(from_start(0xD4)),
};

// TODO: SMB1 file header format

/// A struct that defines the collision header format for Monkey Ball stagedef files.
/// Importantly, this struct stores the offsets as relative offsets from the start of the collision
/// header. We have to construct this after we know where the header begins in the file.
struct StageDefCollisionHeaderFormat {
    center_of_rotation_offset: FileOffset,
    // TODO: Fill out
}

impl StageDefCollisionHeaderFormat {
    fn new(game: &Game, header_start: u32) -> Self {
        match game {
            SMB2 => Self {
                center_of_rotation_offset: FileOffset::OffsetOnly(from_relative(header_start, 0)),
            },
        }
    }
}

// TODO: SMB1 collision header format

pub struct StageDefReader<R: Read + Seek> {
    reader: R,
    game: Game,
}

impl<R: Read + Seek> StageDefReader<R> {
    pub fn new(reader: R, game: &Game) -> Self {
        Self {
            reader,
            game: game.clone(),
        }
    }

    pub fn read_stagedef<B: ByteOrder>(&mut self) -> Result<StageDef> {
        let mut stagedef = StageDef::default();

        let file_header = self.read_header::<B>()?;
    
        // Read magic numbers
        if let Ok(_) = self.reader.seek_fileoffset(file_header.magic_number_1_offset) {
            stagedef.magic_number_1 = self.reader.read_f32::<B>()?;
        }

        if let Ok(_) = self.reader.seek_fileoffset(file_header.magic_number_2_offset) {
            stagedef.magic_number_2 = self.reader.read_f32::<B>()?;
        }

        // TODO: Collision header
        
        // Start position and fallout level
        // TODO: Support multiple start positions
        if let Ok(_) = self.reader.seek_fileoffset(file_header.start_position_ptr_offset) {
            stagedef.start_position = self.reader.read_vec3::<B>()?;
        }

        if let Ok(_) = self.reader.seek_fileoffset(file_header.fallout_position_ptr_offset) {
            stagedef.fallout_level = self.reader.read_f32::<B>()?;
        }

        // Goal list 
        if let FileOffset::CountOffset(c, o) = file_header.goal_list_offset {
            self.reader.seek(o)?;
            for i in 0..c {
                stagedef.goals.push(GlobalStagedefObject::new(self.reader.read_goal::<B>()?, i));
            }
        }

        Ok(stagedef)
    }

    fn read_header<B: ByteOrder>(&mut self) -> Result<StageDefFileHeaderFormat> {
        let default_format = match self.game {
            //TODO: Implement SMB1 support
            Game::SMB1 => unimplemented!(),
            Game::SMB2 | Game::SMBDX => SMB2_FILE_HEADER_FORMAT,
        };

        let mut current_format = StageDefFileHeaderFormat::default();

        // Read magic number offsets
        current_format.magic_number_1_offset = default_format.magic_number_1_offset;
        current_format.magic_number_2_offset = default_format.magic_number_2_offset;

        // Read collision header count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.collision_header_list_offset {
            self.reader.seek(offset)?;
            current_format.collision_header_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read start position offset
        if let FileOffset::OffsetOnly(offset) = default_format.start_position_ptr_offset {
            self.reader.seek(offset)?;
            current_format.start_position_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read fallout level offset
        if let FileOffset::OffsetOnly(offset) = default_format.fallout_position_ptr_offset {
            self.reader.seek(offset)?;
            current_format.fallout_position_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read goal count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.goal_list_offset {
            self.reader.seek(offset)?;
            current_format.goal_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read bumper count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.bumper_list_offset {
            self.reader.seek(offset)?;
            current_format.bumper_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read jamabar count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.jamabar_list_offset {
            self.reader.seek(offset)?;
            current_format.jamabar_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read banana count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.banana_list_offset {
            self.reader.seek(offset)?;
            current_format.banana_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read cone_col count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.cone_col_list_offset {
            self.reader.seek(offset)?;
            current_format.cone_col_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read cyl_col count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.cyl_col_list_offset {
            self.reader.seek(offset)?;
            current_format.cyl_col_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fallout_vol count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.fallout_vol_list_offset {
            self.reader.seek(offset)?;
            current_format.fallout_vol_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read bg_model count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.bg_model_list_offset {
            self.reader.seek(offset)?;
            current_format.bg_model_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fg_model count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.fg_model_list_offset {
            self.reader.seek(offset)?;
            current_format.fg_model_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read reflective_model count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.reflective_model_list_offset {
            self.reader.seek(offset)?;
            current_format.reflective_model_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read model_instance_list count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.model_instance_list_offset {
            self.reader.seek(offset)?;
            current_format.model_instance_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read model_ptr_a count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.model_ptr_a_list_offset {
            self.reader.seek(offset)?;
            current_format.model_ptr_a_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read model_ptr_b count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.model_ptr_b_list_offset {
            self.reader.seek(offset)?;
            current_format.model_ptr_b_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read switch count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.switch_list_offset {
            self.reader.seek(offset)?;
            current_format.switch_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fog_anim_ptr offset
        if let FileOffset::OffsetOnly(offset) = default_format.fog_anim_ptr_offset {
            self.reader.seek(offset)?;
            current_format.fog_anim_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read wormhole count/offset
        if let FileOffset::OffsetOnly(offset) = default_format.wormhole_list_offset {
            self.reader.seek(offset)?;
            current_format.wormhole_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fog_ptr offset
        if let FileOffset::OffsetOnly(offset) = default_format.fog_ptr_offset {
            self.reader.seek(offset)?;
            current_format.fog_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read mystery_3_ptr offset
        if let FileOffset::OffsetOnly(offset) = default_format.mystery_3_ptr_offset {
            self.reader.seek(offset)?;
            current_format.mystery_3_ptr_offset = self.reader.read_offset::<B>()?;
        }

        Ok(current_format)
    }

    // TODO: SMB1 format
    fn read_collision_header<T: ByteOrder, RW: Read + Seek>(reader: &mut RW) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
/// Returns a valid SMB2 main game stagedef with all fields used.
///
/// The fields used by the stagedef are as follows:
///
/// * Magic numbers: 0.0, 1,000.0
/// * Collision headers: 0 at offset 0 TODO
/// * Start position: Offset 0x89c
/// * Fallout position: Offset 0x8b0
/// * Goal list: Offset 0x8b4
/// * TODO: ...
/// * Start position: Vec3: 0.0, 2.75, 14.0, ShortVector3: 0, 0, 0
/// * Fallout level: -20.0
/// * Goal #1: Position 0.0, 0.0, -115.0, Rotation 0, 0, 0, type: blue
fn test_smb2_stagedef_header<T: ByteOrder>() -> Result<Cursor<Vec<u8>>> {
    use byteorder::WriteBytesExt;

    let mut cur = Cursor::new(vec![0; 0x1000]);

    // magic numbers
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x447A0000, 4)?;

    // collision header
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x00000000, 4)?;

    // start position offset
    cur.write_uint::<T>(0x0000089C, 4)?;

    // fallout position offset
    cur.write_uint::<T>(0x000008B0, 4)?;

    // goal list count/offset
    cur.write_uint::<T>(0x00000003, 4)?;
    cur.write_uint::<T>(0x000008B4, 4)?;

    cur.seek(from_start(0x89C))?;

    // start position
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x40300000, 4)?;
    cur.write_uint::<T>(0x41600000, 4)?;

    // start rotation
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x00000000, 4)?;

    cur.seek(from_start(0x8B0))?;

    // fallout level
    cur.write_uint::<T>(0xC1A00000, 4)?;

    cur.seek(from_start(0x8B4))?;

    // goal list
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0xC2E60000, 4)?;
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x00000001, 4)?;

    Ok(cur)
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
fn test_magic_numbers() {
    let file = test_smb2_stagedef_header::<BigEndian>().unwrap();
    let mut sd_reader = StageDefReader::new(file, &Game::SMB2);
    let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

    assert_eq!(stagedef.magic_number_1, 0.0, "BigEndian");
    assert_eq!(stagedef.magic_number_2, 1000.0, "BigEndian");

    let file = test_smb2_stagedef_header::<LittleEndian>().unwrap();
    let mut sd_reader = StageDefReader::new(file, &Game::SMB2);
    let stagedef = sd_reader.read_stagedef::<LittleEndian>().unwrap();

    assert_eq!(stagedef.magic_number_1, 0.0, "LittleEndian");
    assert_eq!(stagedef.magic_number_2, 1000.0, "LittleEndian");
}

#[test]
fn test_start_fallout_pos_parse() {
    let expected_pos = Vector3 {
        x: 0.0,
        y: 2.75,
        z: 14.0,
    };
    let expected_rot = ShortVector3 { x: 0, y: 0, z: 0 };
    let expected_flevel = -20.0;

    let file = test_smb2_stagedef_header::<BigEndian>().unwrap();
    let mut sd_reader = StageDefReader::new(file, &Game::SMB2);
    let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

    assert_eq!(stagedef.start_position, expected_pos, "BigEndian");
    assert_eq!(stagedef.start_rotation, expected_rot, "BigEndian");
    assert_eq!(stagedef.fallout_level, expected_flevel, "BigEndian");

    let file = test_smb2_stagedef_header::<LittleEndian>().unwrap();
    let mut sd_reader = StageDefReader::new(file, &Game::SMB2);
    let stagedef = sd_reader.read_stagedef::<LittleEndian>().unwrap();

    assert_eq!(stagedef.start_position, expected_pos, "LittleEndian");
    assert_eq!(stagedef.start_rotation, expected_rot, "LittleEndian");
    assert_eq!(stagedef.fallout_level, expected_flevel, "LittleEndian");
}

#[test]
fn test_goal_parse() {
    let expected_goal = Goal {
        position: Vector3 {
            x: 0.0,
            y: 0.0,
            z: -115.0,
        },
        rotation: ShortVector3 { x: 0, y: 0, z: 0 },
        goal_type: GoalType::Blue,
    };

    let file = test_smb2_stagedef_header::<BigEndian>().unwrap();
    let mut sd_reader = StageDefReader::new(file, &Game::SMB2);
    let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

    assert_eq!(*stagedef.goals[0].object.lock().unwrap(), expected_goal);
}
