//! Handles parsing of an uncompressed Monkey Ball stage binary.
use crate::stagedef::{
    Game, GlobalStagedefObject, Goal, GoalType, ShortVector3, StageDef, Vector3,
};
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::{
    io::{self, BufReader, BufWriter, Cursor, Error, Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

const fn from_start(offset: u32) -> Option<SeekFrom> {
    Some(SeekFrom::Start(offset as u64))
}

trait ReadBytesExtSmb {
    fn read_vec3<U: ByteOrder>(&mut self) -> Result<Vector3, Error>;
    fn read_vec3_short<U: ByteOrder>(&mut self) -> Result<ShortVector3, Error>;
    fn read_count_offset<U: ByteOrder>(&mut self) -> Result<CountOffset, Error>;
    fn read_goal<U: ByteOrder>(&mut self) -> Result<Goal, Error>;
}

impl<T: ReadBytesExt> ReadBytesExtSmb for T {
    fn read_vec3<U: ByteOrder>(&mut self) -> Result<Vector3, Error> {
        let x = self.read_f32::<U>()?;
        let y = self.read_f32::<U>()?;
        let z = self.read_f32::<U>()?;

        Ok(Vector3 { x, y, z })
    }

    fn read_vec3_short<U: ByteOrder>(&mut self) -> Result<ShortVector3, Error> {
        let x = self.read_u16::<U>()?;
        let y = self.read_u16::<U>()?;
        let z = self.read_u16::<U>()?;

        Ok(ShortVector3 { x, y, z })
    }

    fn read_count_offset<U: ByteOrder>(&mut self) -> Result<CountOffset, Error> {
        let count = self.read_u32::<U>()?;
        let offset = self.read_u32::<U>()?;

        Ok(CountOffset { count, offset })
    }

    fn read_goal<U: ByteOrder>(&mut self) -> Result<Goal, Error> {
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

impl StageDef {
    pub fn read_stagedef<T: ByteOrder, RW: Read + Seek>(
        file: &mut RW,
        game: &Game,
    ) -> io::Result<StageDef> {
        let mut stagedef = StageDef::default();
        //let mut removethis_writer = BufWriter::new(file);
        let format = match game {
            //TODO: Implement SMB1 support
            Game::SMB1 => SMB2_FORMAT,
            Game::SMB2 | Game::SMBDX => SMB2_FORMAT,
        };

        StageDef::read_header::<T, RW>(&mut stagedef, file, &format)?;

        Ok(stagedef)
    }

    fn read_offset_and_seek<T: ByteOrder, RW: Read + Seek>(
        reader: &mut RW,
        offset: SeekFrom,
    ) -> io::Result<()> {
        reader.seek(offset)?;
        let ptr = from_start(reader.read_u32::<T>()?);
        reader.seek(ptr.unwrap())?;
        Ok(())
    }

    fn read_header<T: ByteOrder, RW: Read + Seek>(
        stagedef: &mut StageDef,
        reader: &mut RW,
        format: &StageDefFormat,
    ) -> io::Result<()> {
        // Read magic numbers
        reader.seek(format.magic_number_1_offset.unwrap())?;
        stagedef.magic_number_1 = reader.read_f32::<T>()?;

        reader.seek(format.magic_number_2_offset.unwrap())?;
        stagedef.magic_number_2 = reader.read_f32::<T>()?;

        // Read start pos
        StageDef::read_offset_and_seek::<T, RW>(reader, format.start_position_ptr_offset.unwrap())?;
        stagedef.start_position = reader.read_vec3::<T>()?;
        stagedef.start_rotation = reader.read_vec3_short::<T>()?;

        // Read fallout pos
        StageDef::read_offset_and_seek::<T, RW>(
            reader,
            format.fallout_position_ptr_offset.unwrap(),
        )?;
        stagedef.fallout_level = reader.read_f32::<T>()?;

        // Read goal list
        reader.seek(format.goal_list_offset.unwrap())?;
        let goal_list_co = reader.read_count_offset::<T>()?;
        reader.seek(from_start(goal_list_co.offset).unwrap())?;
        for i in 0..goal_list_co.count {
            let goal = reader.read_goal::<T>()?;
            stagedef.goals.push(GlobalStagedefObject {
                object: Arc::new(Mutex::new(goal)),
                index: i,
            });
        }

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
fn test_smb2_stagedef_header<T: ByteOrder>() -> io::Result<Cursor<Vec<u8>>> {
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

    cur.seek(from_start(0x89C).unwrap())?;

    // start position
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x40300000, 4)?;
    cur.write_uint::<T>(0x41600000, 4)?;

    // start rotation
    cur.write_uint::<T>(0x00000000, 4)?;
    cur.write_uint::<T>(0x00000000, 4)?;

    cur.seek(from_start(0x8B0).unwrap())?;

    // fallout level
    cur.write_uint::<T>(0xC1A00000, 4)?;

    cur.seek(from_start(0x8B4).unwrap())?;

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
    let mut file = test_smb2_stagedef_header::<BigEndian>().unwrap();
    let stagedef = StageDef::read_stagedef::<BigEndian, _>(&mut file, &Game::SMB2).unwrap();

    assert_eq!(stagedef.magic_number_1, 0.0, "BigEndian");
    assert_eq!(stagedef.magic_number_2, 1000.0, "BigEndian");

    let mut file = test_smb2_stagedef_header::<LittleEndian>().unwrap();
    let stagedef = StageDef::read_stagedef::<LittleEndian, _>(&mut file, &Game::SMB2).unwrap();

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

    let mut file = test_smb2_stagedef_header::<BigEndian>().unwrap();
    let stagedef = StageDef::read_stagedef::<BigEndian, _>(&mut file, &Game::SMB2).unwrap();

    assert_eq!(stagedef.start_position, expected_pos, "BigEndian");
    assert_eq!(stagedef.start_rotation, expected_rot, "BigEndian");
    assert_eq!(stagedef.fallout_level, expected_flevel, "BigEndian");

    let mut file = test_smb2_stagedef_header::<LittleEndian>().unwrap();
    let stagedef = StageDef::read_stagedef::<LittleEndian, _>(&mut file, &Game::SMB2).unwrap();

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

    let mut file = test_smb2_stagedef_header::<BigEndian>().unwrap();
    let stagedef = StageDef::read_stagedef::<BigEndian, _>(&mut file, &Game::SMB2).unwrap();

    assert_eq!(stagedef.goals[0], expected_goal);
}
