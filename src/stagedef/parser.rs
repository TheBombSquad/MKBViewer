//! Handles parsing of an uncompressed Monkey Ball stage binary.
use crate::stagedef::common::{
    Game, GlobalStagedefObject, ShortVector3, StageDef, StageDefObject, StageDefParsable, Vector3,
};
use crate::stagedef::objects::*;
use anyhow::Result;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write},
};
use tracing::{debug, event, warn, Level};

/// Helper function that returns a new [``SeekFrom::Start``] from the given [``u32``] offset.
///
/// Mostly used for convenience for writing out default header formats.
const fn from_start(offset: u64) -> SeekFrom {
    SeekFrom::Start(offset)
}

/// Helper function that takes a [``SeekFrom::Start``] and applies the given [``u32``] offset to it.
///
/// Mostly used for convenience for header formats like collision headers.
/// Does not work on other variants of [``SeekFrom``].
const fn from_relative(start: SeekFrom, offset: u32) -> SeekFrom {
    if let SeekFrom::Start(o) = start {
        SeekFrom::Start(o + offset as u64)
    } else {
        panic!("Did not pass a SeekFrom::Start to from_relative");
    }
}

/// Helper function that takes two [``SeekFrom::Start``] objects, and subtracts their offsets.
///
/// Does not work on other variants of [``SeekFrom``].
/// Returns [``Err``] if the resulting value would be negative.
fn try_get_offset_difference(x: &SeekFrom, y: &SeekFrom) -> Result<u32> {
    if let SeekFrom::Start(x_offset) = x {
        if let SeekFrom::Start(y_offset) = y {
            if y_offset > x_offset {
                Err(anyhow::Error::msg("Resulting offset difference was negative"))
            } else {
                Ok(u32::try_from(*x_offset).unwrap() - u32::try_from(*y_offset).unwrap())
            }
        } else {
            panic!("Did not pass a SeekFrom::Start to y parameter for difference");
        }
    } else {
        panic!("Did not pass a SeekFrom::Start to x parameter for difference");
    }
}

/// Defines possible file offset types within a [``StageDef``].
#[derive(Default, Clone, Copy, Debug)]
pub enum FileOffset {
    /// The offset, or the structure it refers to, does not exist in this format. Nothing will be read.
    #[default]
    Unused,
    /// The offset points to a single structure.
    OffsetOnly(SeekFrom),
    /// The offset points to multiple structures in a contiguous list.
    CountOffset(u32, SeekFrom),
}

/// Extends [``ReadBytesExt``] with methods for reading common [``StageDef``] types.
pub trait ReadBytesExtSmb: ReadBytesExt + Seek {
    fn read_vec3<U: ByteOrder>(&mut self) -> Result<Vector3>;
    fn read_vec3_short<U: ByteOrder>(&mut self) -> Result<ShortVector3>;
    fn read_offset<U: ByteOrder>(&mut self) -> Result<FileOffset>;
    fn read_count_offset<U: ByteOrder>(&mut self) -> Result<FileOffset>;
    fn read_model_name_from_offset<U: ByteOrder>(&mut self) -> Result<String>;
}

impl<T: ReadBytesExt + Seek> ReadBytesExtSmb for T {
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
        let offset = from_start(u64::from(self.read_u32::<U>()?));

        Ok(FileOffset::OffsetOnly(offset))
    }

    fn read_count_offset<U: ByteOrder>(&mut self) -> Result<FileOffset> {
        let count = self.read_u32::<U>()?;
        let offset = self.read_u32::<U>()?;

        if count == 0 || offset == 0 {
            Ok(FileOffset::Unused)
        } else {
            Ok(FileOffset::CountOffset(count, from_start(u64::from(offset))))
        }
    }

    fn read_model_name_from_offset<U: ByteOrder>(&mut self) -> Result<String> {
        let name_offset = from_start(u64::from(self.read_u32::<U>()?));
        let return_position = from_start(self.stream_position()?);

        self.seek(name_offset)?;
        
        let mut u8_arr: Vec<char> = Vec::new();
        let mut current_byte = 0xFF;
        while current_byte != 0x0 {
            current_byte = self.read_u8()?;
            u8_arr.push(current_byte as char);
        }

        self.seek(return_position)?;

        let string = u8_arr.iter().collect::<String>();
        Ok(string)
    }
}

/// Extends [``std::io::Seek``] with a method for attempting to seek to a [``FileOffset``].
trait SeekExtSmb {
    fn try_seek(&mut self, offset: FileOffset) -> io::Result<u64>;
}

impl<T: Seek> SeekExtSmb for T {
    fn try_seek(&mut self, offset: FileOffset) -> io::Result<u64> {
        match offset {
            FileOffset::Unused => Err(io::Error::new(io::ErrorKind::Other, "Attempted to seek to an unused value")),
            FileOffset::OffsetOnly(o) | FileOffset::CountOffset(_, o) => self.seek(o),
        }
    }
}

/// Defines the file header format for a Monkey Ball stagedef file.
///
/// The fields define the location from the start of the file in which the given structure can be
/// found. These fields are optional, for situations where certain structures are not in a
/// particular game (for example, Super Monkey Ball 1 does not have wormholes).
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

/// Defines the collision header format for Monkey Ball stagedef files.
///
/// This struct stores the offsets as relative offsets from the start of the collision
/// header. We have to construct this after we know where the header begins in the file.
struct StageDefCollisionHeaderFormat {
    center_of_rotation_offset: FileOffset,
    initial_rotation_offset: FileOffset,
    animation_type_offset: FileOffset,
    animation_header_ptr_offset: FileOffset,
    conveyor_vector_offset: FileOffset,
    collision_triangle_list_offset: FileOffset,
    collision_grid_triangle_list_offset: FileOffset,
    collision_grid_start_x_offset: FileOffset,
    collision_grid_start_z_offset: FileOffset,
    collision_grid_step_x_offset: FileOffset,
    collision_grid_step_z_offset: FileOffset,
    collision_grid_step_x_count_offset: FileOffset,
    collision_grid_step_z_count_offset: FileOffset,
    goal_list_offset: FileOffset,
    bumper_list_offset: FileOffset,
    jamabar_list_offset: FileOffset,
    banana_list_offset: FileOffset,
    cone_col_list_offset: FileOffset,
    sphere_col_list_offset: FileOffset,
    cyl_col_list_offset: FileOffset,
    fallout_vol_list_offset: FileOffset,
    reflective_model_list_offset: FileOffset,
    model_instance_list_offset: FileOffset,
    model_ptr_b_list_offset: FileOffset,
    unk0x9c_offset: FileOffset,
    unk0xa0_offset: FileOffset,
    animation_id_offset: FileOffset,
    unk0xa6_offset: FileOffset,
    switch_list_offset: FileOffset,
    unk0xb0_offset: FileOffset,
    mystery_5_offset: FileOffset,
    seesaw_sensitivity_offset: FileOffset,
    seesaw_friction_offset: FileOffset,
    seesaw_spring_offset: FileOffset,
    wormhole_list_offset: FileOffset,
    animation_state_init_offset: FileOffset,
    unk0xd0_offset: FileOffset,
    animation_loop_point_offset: FileOffset,
    texture_scroll_ptr_offset: FileOffset,
}

impl StageDefCollisionHeaderFormat {
    #[rustfmt::skip]
    fn new(game: Game, header_start: SeekFrom) -> Self {
        match game {
            SMB2 => Self {
                center_of_rotation_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x0)),
                initial_rotation_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xC)),
                animation_type_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x12)),
                animation_header_ptr_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x14)),
                conveyor_vector_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x18)),
                collision_triangle_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x24)),
                collision_grid_triangle_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x28)),
                collision_grid_start_x_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x2C)),
                collision_grid_start_z_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x30)),
                collision_grid_step_x_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x34)),
                collision_grid_step_z_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x38)),
                collision_grid_step_x_count_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x3C)),
                collision_grid_step_z_count_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x40)),
                goal_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x44)),
                bumper_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x4C)),
                jamabar_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x54)),
                banana_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x5C)),
                cone_col_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x64)),
                sphere_col_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x6C)),
                cyl_col_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x74)),
                fallout_vol_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x7C)),
                reflective_model_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x84)),
                model_instance_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x8C)),
                model_ptr_b_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x94)),
                unk0x9c_offset: FileOffset::OffsetOnly(from_relative(header_start, 0x9C)),
                unk0xa0_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xA0)),
                animation_id_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xA4)),
                unk0xa6_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xA6)),
                switch_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xA8)),
                unk0xb0_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xB0)),
                mystery_5_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xB4)),
                seesaw_sensitivity_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xB8)),
                seesaw_friction_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xBC)),
                seesaw_spring_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xC0)),
                wormhole_list_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xC4)),
                animation_state_init_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xCC)),
                unk0xd0_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xD0)),
                animation_loop_point_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xD4)),
                texture_scroll_ptr_offset: FileOffset::OffsetOnly(from_relative(header_start, 0xD8)),
            },
        }
    }
}

/// Handles reading a stagedef with a given reader, game type, and format.
// TODO: SMB1 collision header format
pub struct StageDefReader<R: Read + Seek> {
    reader: R,
    game: Game,
    file_header: StageDefFileHeaderFormat,
}

impl<R: Read + Seek> StageDefReader<R> {
    pub fn new(reader: R, game: Game) -> Self {
        Self {
            reader,
            game,
            file_header: StageDefFileHeaderFormat::default(),
        }
    }

    // Read in a new StageDef from our reader.
    pub fn read_stagedef<B: ByteOrder>(&mut self) -> Result<StageDef> {
        let mut stagedef = StageDef::default();

        self.file_header = self.read_file_header_offsets::<B>()?;

        // Read magic numbers
        if self.reader.try_seek(self.file_header.magic_number_1_offset).is_ok() {
            stagedef.magic_number_1 = self.reader.read_f32::<B>()?;
        }

        if self.reader.try_seek(self.file_header.magic_number_2_offset).is_ok() {
            stagedef.magic_number_2 = self.reader.read_f32::<B>()?;
        }

        // Read start position and fallout level
        // TODO: Support multiple start positions
        if self.reader.try_seek(self.file_header.start_position_ptr_offset).is_ok() {
            stagedef.start_position = self.reader.read_vec3::<B>()?;
        }

        if self.reader.try_seek(self.file_header.fallout_position_ptr_offset).is_ok() {
            stagedef.fallout_level = self.reader.read_f32::<B>()?;
        }

        // TODO:: Fill this out...

        // Read goal list
        if let Ok(goals) = self.read_stagedef_list::<B, Goal>(self.file_header.goal_list_offset) {
            stagedef.goals = goals;
        }

        // Read bumper list
        if let Ok(bumpers) = self.read_stagedef_list::<B, Bumper>(self.file_header.bumper_list_offset) {
            stagedef.bumpers = bumpers;
        }

        // Read jamabar list
        if let Ok(jamabars) = self.read_stagedef_list::<B, Jamabar>(self.file_header.jamabar_list_offset) {
            stagedef.jamabars = jamabars;
        }

        // Read banana list
        if let Ok(bananas) = self.read_stagedef_list::<B, Banana>(self.file_header.banana_list_offset) {
            stagedef.bananas = bananas;
        }

        // Read cone_col list
        if let Ok(cone_cols) = self.read_stagedef_list::<B, ConeCollision>(self.file_header.cone_col_list_offset) {
            stagedef.cone_collisions = cone_cols;
        }

        // Read sphere_col list
        if let Ok(sphere_cols) = self.read_stagedef_list::<B, SphereCollision>(self.file_header.sphere_col_list_offset) {
            stagedef.sphere_collisions = sphere_cols;
        }

        // Read cyl_col list
        if let Ok(cyl_cols) = self.read_stagedef_list::<B, CylinderCollision>(self.file_header.cyl_col_list_offset) {
            stagedef.cylinder_collisions = cyl_cols;
        }

        // Read fallout_vol list
        if let Ok(fallout_vols) = self.read_stagedef_list::<B, FalloutVolume>(self.file_header.fallout_vol_list_offset) {
            stagedef.fallout_volumes = fallout_vols;
        }

        // Read background_model list
        if let Ok(background_models) = self.read_stagedef_list::<B, BackgroundModel>(self.file_header.bg_model_list_offset) {
            stagedef.background_models = background_models;
        }

        // Read all collision headers - done last so we can properly set up references to other global
        // stagedef objects
        // TODO: Change based on game
        if let FileOffset::CountOffset(c, o) = self.file_header.collision_header_list_offset {
            for i in 0..c {
                let current_offset = from_relative(o, CollisionHeader::get_size() * i);
                self.reader.seek(current_offset)?;

                stagedef
                    .collision_headers
                    .push(self.read_collision_header::<B>(&stagedef, current_offset)?);
            }
        }
        Ok(stagedef)
    }

    // Determine the default format based on our reader's Game attribute, then use the default format
    // to parse the stagedef's offsets.
    fn read_file_header_offsets<B: ByteOrder>(&mut self) -> Result<StageDefFileHeaderFormat> {
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
        if self.reader.try_seek(default_format.collision_header_list_offset).is_ok() {
            current_format.collision_header_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read start position offset
        if self.reader.try_seek(default_format.start_position_ptr_offset).is_ok() {
            current_format.start_position_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read fallout level offset
        if self.reader.try_seek(default_format.fallout_position_ptr_offset).is_ok() {
            current_format.fallout_position_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read goal count/offset
        if self.reader.try_seek(default_format.goal_list_offset).is_ok() {
            current_format.goal_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read bumper count/offset
        if self.reader.try_seek(default_format.bumper_list_offset).is_ok() {
            current_format.bumper_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read jamabar count/offset
        if self.reader.try_seek(default_format.jamabar_list_offset).is_ok() {
            current_format.jamabar_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read banana count/offset
        if self.reader.try_seek(default_format.banana_list_offset).is_ok() {
            current_format.banana_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read cone_col count/offset
        if self.reader.try_seek(default_format.cone_col_list_offset).is_ok() {
            current_format.cone_col_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read cyl_col count/offset
        if self.reader.try_seek(default_format.cyl_col_list_offset).is_ok() {
            current_format.cyl_col_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fallout_vol count/offset
        if self.reader.try_seek(default_format.fallout_vol_list_offset).is_ok() {
            current_format.fallout_vol_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read bg_model count/offset
        if self.reader.try_seek(default_format.bg_model_list_offset).is_ok() {
            current_format.bg_model_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fg_model count/offset
        if self.reader.try_seek(default_format.fg_model_list_offset).is_ok() {
            current_format.fg_model_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read reflective_model count/offset
        if self.reader.try_seek(default_format.reflective_model_list_offset).is_ok() {
            current_format.reflective_model_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read model_instance_list count/offset
        if self.reader.try_seek(default_format.model_instance_list_offset).is_ok() {
            current_format.model_instance_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read model_ptr_a count/offset
        if self.reader.try_seek(default_format.model_ptr_a_list_offset).is_ok() {
            current_format.model_ptr_a_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read model_ptr_b count/offset
        if self.reader.try_seek(default_format.model_ptr_b_list_offset).is_ok() {
            current_format.model_ptr_b_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read switch count/offset
        if self.reader.try_seek(default_format.switch_list_offset).is_ok() {
            current_format.switch_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fog_anim_ptr offset
        if self.reader.try_seek(default_format.fog_anim_ptr_offset).is_ok() {
            current_format.fog_anim_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read wormhole count/offset
        if self.reader.try_seek(default_format.wormhole_list_offset).is_ok() {
            current_format.wormhole_list_offset = self.reader.read_count_offset::<B>()?;
        }

        // Read fog_ptr offset
        if self.reader.try_seek(default_format.fog_ptr_offset).is_ok() {
            current_format.fog_ptr_offset = self.reader.read_offset::<B>()?;
        }

        // Read mystery_3_ptr offset
        if self.reader.try_seek(default_format.mystery_3_ptr_offset).is_ok() {
            current_format.mystery_3_ptr_offset = self.reader.read_offset::<B>()?;
        }

        Ok(current_format)
    }

    // TODO: SMB1 format
    // Reads a collision header from the specified offset. Does not advance the reader by the max
    // size of a collision header, 0x49C.
    fn read_collision_header<B: ByteOrder>(&mut self, stagedef: &StageDef, offset: SeekFrom) -> Result<CollisionHeader> {
        let current_format = StageDefCollisionHeaderFormat::new(self.game, offset);
        let mut collision_header = CollisionHeader::default();

        // Read center of rotation position
        if self.reader.try_seek(current_format.center_of_rotation_offset).is_ok() {
            collision_header.center_of_rotation_position = self.reader.read_vec3::<B>()?;
        }

        // TODO: Fill out the rest of the collision header structs
        // Read goals
        if let Ok(goals) = self.read_local_object_list::<B, Goal>(
            current_format.goal_list_offset,
            self.file_header.goal_list_offset,
            &stagedef.goals,
        ) {
            collision_header.goals = goals;
        }

        // Read bumpers
        if let Ok(bumpers) = self.read_local_object_list::<B, Bumper>(
            current_format.bumper_list_offset,
            self.file_header.bumper_list_offset,
            &stagedef.bumpers,
        ) {
            collision_header.bumpers = bumpers;
        }

        // Read jamabars
        if let Ok(jamabars) = self.read_local_object_list::<B, Jamabar>(
            current_format.jamabar_list_offset,
            self.file_header.jamabar_list_offset,
            &stagedef.jamabars,
        ) {
            collision_header.jamabars = jamabars;
        }

        // Read bananas
        if let Ok(bananas) = self.read_local_object_list::<B, Banana>(
            current_format.banana_list_offset,
            self.file_header.banana_list_offset,
            &stagedef.bananas,
        ) {
            collision_header.bananas = bananas;
        }

        // Read cone_collisions
        if let Ok(cone_collisions) = self.read_local_object_list::<B, ConeCollision>(
            current_format.cone_col_list_offset,
            self.file_header.cone_col_list_offset,
            &stagedef.cone_collisions,
        ) {
            collision_header.cone_collisions = cone_collisions;
        }

        // Read sphere_collisions
        if let Ok(sphere_collisions) = self.read_local_object_list::<B, SphereCollision>(
            current_format.sphere_col_list_offset,
            self.file_header.sphere_col_list_offset,
            &stagedef.sphere_collisions,
        ) {
            collision_header.sphere_collisions = sphere_collisions;
        }

        // Read cylinder_collisions
        if let Ok(cylinder_collisions) = self.read_local_object_list::<B, CylinderCollision>(
            current_format.cyl_col_list_offset,
            self.file_header.cyl_col_list_offset,
            &stagedef.cylinder_collisions,
        ) {
            collision_header.cylinder_collisions = cylinder_collisions;
        }

        // Read fallout_volumes
        if let Ok(fallout_volumes) = self.read_local_object_list::<B, FalloutVolume>(
            current_format.fallout_vol_list_offset,
            self.file_header.fallout_vol_list_offset,
            &stagedef.fallout_volumes,
        ) {
            collision_header.fallout_volumes = fallout_volumes;
        }

        // Read background_model list
        if let Ok(background_models) = self.read_stagedef_list::<B, BackgroundModel>(self.file_header.bg_model_list_offset) {
            collision_header.background_models = background_models;
        }

        Ok(collision_header)
    }

    /// Read a global stagedef object list
    fn read_stagedef_list<B: ByteOrder, T: StageDefParsable>(
        &mut self,
        offset: FileOffset,
    ) -> Result<Vec<GlobalStagedefObject<T>>> {
        if let FileOffset::CountOffset(c, o) = offset {
            let mut vec = Vec::new();
            self.reader.seek(o)?;
            for i in 0..c {
                let read_obj = T::try_from_reader::<R, B>(&mut self.reader);

                match read_obj {
                    Ok(obj) => vec.push(GlobalStagedefObject::new(obj, i)),
                    Err(err) => warn!("{err}"),
                }
            }
            Ok(vec)
        } else {
            Err(anyhow::Error::msg("No object list was read"))
        }
    }

    /// Return all objects found within a local stagedef list
    ///
    /// This is often a subset of a global list, so we pass the relevant global list to this
    /// function in order to determine which objects we should return.
    fn read_local_object_list<B: ByteOrder, T: StageDefParsable>(
        &mut self,
        offset: FileOffset,
        global_list_offset: FileOffset,
        global_list: &[GlobalStagedefObject<T>],
    ) -> Result<Vec<GlobalStagedefObject<T>>> {
        if self.reader.try_seek(offset).is_ok() {
            let local_count_offset = self.reader.read_count_offset::<B>()?;
            if let FileOffset::CountOffset(local_count, local_offset) = local_count_offset {
                self.reader.seek(local_offset)?;

                // Attempt to get objects from global list and re-adjust indices for our local list
                let vec = match Self::get_global_objs_from_local_list(local_count, &local_offset, &global_list_offset, global_list) {
                    Some(objs) => objs,
                    None => self.read_stagedef_list::<B, T>(local_count_offset)?,
                };

                Ok(vec)
            } else {
                Err(anyhow::Error::msg("No object list was read - no local offset"))
            }
        } else {
            Err(anyhow::Error::msg("No object list was read - could not seek to given offset"))
        }
    }

    /// Return the intersection between a local and global stagedef object list, or ``None`` if no
    /// overlap exists.
    fn get_global_objs_from_local_list<T: StageDefParsable>(
        local_count: u32,
        local_offset: &SeekFrom,
        global_co: &FileOffset,
        global_obj_list: &[GlobalStagedefObject<T>],
    ) -> Option<Vec<GlobalStagedefObject<T>>> {
        if let FileOffset::CountOffset(global_count, global_offset) = global_co {
            // We want to compare the local offset of this list to the global one to find out
            // where we are in the global list
            if let Ok(diff) = try_get_offset_difference(local_offset, global_offset) {
                // The difference isn't negative, so the object(s) is likely to be in or after the
                // global list
                let global_size = global_count * T::get_size();
                // The difference is within the bounds of the list
                if diff < global_size {
                    // Get the global starting index for the local list
                    let global_start_index = diff / T::get_size();
                    let mut local_reindex_value = 0;
                    let matching_global_objs: Vec<GlobalStagedefObject<T>> = global_obj_list
                        .iter()
                        .filter(|global| global.index >= global_start_index)
                        .take(local_count as usize)
                        .cloned()
                        .map(|mut local| {
                            local.index = local_reindex_value;
                            local_reindex_value += 1;
                            local
                        })
                        .collect();
                    Some(matching_global_objs)
                }
                // The difference isn't within the bounds of the list, so the object(s) is not in
                // the global list
                else {
                    warn!(
                        "Failed global object retrieval for type {}: local list of size {:} larger than global list of size {:}",
                        T::get_name(), diff, global_size
                    );
                    None
                }
            }
            // The difference is negative, so the object(s) is before the global list for some
            // reason
            else {
                warn!(
                    "Failed global object retrieval for type {}: objects before list",
                    T::get_name()
                );
                None
            }
        }
        // There is no global list
        else {
            warn!("Failed global object retrieval for type {}: no global list", T::get_name());
            None
        }
    }
}

mod test {
    #![allow(clippy::unreadable_literal)]
    #![allow(clippy::float_cmp)]
    use super::*;

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
        cur.write_uint::<T>(0x00000001, 4)?;
        cur.write_uint::<T>(0x00001BFC, 4)?;

        // start position offset
        cur.write_uint::<T>(0x0000089C, 4)?;

        // fallout position offset
        cur.write_uint::<T>(0x000008B0, 4)?;

        // goal list count/offset
        cur.write_uint::<T>(0x00000001, 4)?;
        cur.write_uint::<T>(0x000008B4, 4)?;

        // banana list count/offset
        cur.seek(from_start(0x30))?;
        cur.write_uint::<T>(0x00000007, 4)?;
        cur.write_uint::<T>(0x000008C8, 4)?;

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

        // banana list
        cur.seek(from_start(0x8C8));
        cur.write_uint::<T>(0x41500000, 4)?;
        cur.write_uint::<T>(0x3F99999A, 4)?;
        cur.write_uint::<T>(0xC2CC0000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0xC1500000, 4)?;
        cur.write_uint::<T>(0x3F99999A, 4)?;
        cur.write_uint::<T>(0xC2CC0000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0xC1500000, 4)?;
        cur.write_uint::<T>(0x3F99999A, 4)?;
        cur.write_uint::<T>(0xC3000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x41500000, 4)?;
        cur.write_uint::<T>(0x3F99999A, 4)?;
        cur.write_uint::<T>(0xC3000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x41900000, 4)?;
        cur.write_uint::<T>(0x3F99999A, 4)?;
        cur.write_uint::<T>(0xC2E60000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0xC1900000, 4)?;
        cur.write_uint::<T>(0x3F99999A, 4)?;
        cur.write_uint::<T>(0xC2E60000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x3F99999A, 4)?;
        cur.write_uint::<T>(0xC3050000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;

        cur.seek(from_start(0x1BFC))?;

        // collision header #1
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00002098, 4)?;
        cur.write_uint::<T>(0x000119E4, 4)?;
        cur.write_uint::<T>(0xC1A92F92, 4)?;
        cur.write_uint::<T>(0xC30825EB, 4)?;
        cur.write_uint::<T>(0x40292F34, 4)?;
        cur.write_uint::<T>(0x413064F2, 4)?;
        cur.write_uint::<T>(0x00000010, 4)?;
        cur.write_uint::<T>(0x00000010, 4)?;
        cur.write_uint::<T>(0x00000001, 4)?;
        cur.write_uint::<T>(0x000008B4, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000007, 4)?;
        cur.write_uint::<T>(0x000008C8, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000004, 4)?;
        cur.write_uint::<T>(0x00000998, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000003, 4)?;
        cur.write_uint::<T>(0x0000098C, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00000000, 4)?;
        cur.write_uint::<T>(0x00001AFC, 4)?;

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
        let mut sd_reader = StageDefReader::new(file, Game::SMB2);
        let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

        assert_eq!(stagedef.magic_number_1, 0.0, "BigEndian");
        assert_eq!(stagedef.magic_number_2, 1000.0, "BigEndian");

        let file = test_smb2_stagedef_header::<LittleEndian>().unwrap();
        let mut sd_reader = StageDefReader::new(file, Game::SMB2);
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
        let mut sd_reader = StageDefReader::new(file, Game::SMB2);
        let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

        assert_eq!(stagedef.start_position, expected_pos, "BigEndian");
        assert_eq!(stagedef.start_rotation, expected_rot, "BigEndian");
        assert_eq!(stagedef.fallout_level, expected_flevel, "BigEndian");

        let file = test_smb2_stagedef_header::<LittleEndian>().unwrap();
        let mut sd_reader = StageDefReader::new(file, Game::SMB2);
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
        let mut sd_reader = StageDefReader::new(file, Game::SMB2);
        let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

        assert_eq!(*stagedef.goals[0].object.lock().unwrap(), expected_goal);
    }

    #[test]
    fn test_banana_parse() {
        let file = test_smb2_stagedef_header::<BigEndian>().unwrap();
        let mut sd_reader = StageDefReader::new(file, Game::SMB2);
        let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

        assert_eq!(stagedef.bananas.len(), 7);
    }

    #[test]
    fn test_collision_header_goal_parse() {
        tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
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
        let mut sd_reader = StageDefReader::new(file, Game::SMB2);
        let stagedef = sd_reader.read_stagedef::<BigEndian>().unwrap();

        assert_eq!(stagedef.collision_headers.len(), 1);
        assert_eq!(stagedef.collision_headers[0].goals.len(), 1);

        let test_goal = stagedef.collision_headers[0].goals[0].object.lock().unwrap();
        assert_eq!(*test_goal, expected_goal);
    }
    #[test]
    fn element_size_test() {
        assert_eq!(true, true);
    }
}
