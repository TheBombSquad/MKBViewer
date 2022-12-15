//! Handles representation of the Monkey Ball stagedef format
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs, io::Cursor};

use byteorder::{BigEndian, LittleEndian};

use crate::app::FileHandleWrapper;

/// Contains a StageDef, as well as extra information about the file
///
/// By default, this will be a big-endian SMB2 stagedef
#[derive(Default)]
pub struct StageDefInstance {
    pub stagedef: StageDef,
    pub game: Game,
    pub endianness: Endianness,
    file: FileHandleWrapper,
}

impl StageDefInstance {
    pub fn new(file: FileHandleWrapper) -> Result<Self, std::io::Error> {
        let game = Game::SMB2;
        let endianness = Endianness::BigEndian;

        let mut reader = file.get_cursor();

        //TODO: Implement endianness/game selection
        let stagedef = match endianness {
            Endianness::BigEndian => {
                StageDef::read_stagedef::<BigEndian, Cursor<Vec<u8>>>(&mut reader, &game)
            }
            Endianness::LittleEndian => {
                StageDef::read_stagedef::<LittleEndian, Cursor<Vec<u8>>>(&mut reader, &game)
            }
        }?;

        Ok(Self {
            stagedef,
            game,
            endianness,
            file,
        })
    }

    pub fn with_endianness(mut self, endianness: Endianness) -> StageDefInstance {
        self.endianness = endianness;
        self
    }

    pub fn with_game(mut self, game: Game) -> StageDefInstance {
        self.game = game;
        self
    }
}

// Common structures/enums

/// 32-bit floating point 3 dimensional vector.
#[derive(Default, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// 16-bit 'short' 3 dimensional vector. Used to represent rotations in Monkey Ball stagedefs.
#[derive(Default, Debug, PartialEq)]
pub struct ShortVector3 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

pub enum Game {
    SMB1,
    SMB2,
    SMBDX,
}

impl Default for Game {
    fn default() -> Self {
        Game::SMB2
    }
}

pub enum Endianness {
    BigEndian,
    LittleEndian,
}

impl Default for Endianness {
    fn default() -> Self {
        Endianness::BigEndian
    }
}

pub enum AnimationState {
    Play,
    Pause,
    Reverse,
    FastForward,
    FastReverse,
}

pub enum AnimationType {
    LoopingAnimation,
    PlayOnceAnimation,
    Seesaw,
}

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq)]
pub enum GoalType {
    Blue = 0x0,
    Green = 0x1,
    Red = 0x2,
}

pub enum BananaType {
    Single,
    Bunch,
}

pub struct CollisionTriangle {
    pub position: Vector3,
    pub normal: Vector3,
    pub rotation: ShortVector3,
    pub delta_x2_x1: f32,
    pub delta_y2_y1: f32,
    pub delta_x3_x1: f32,
    pub delta_y3_y1: f32,
    pub x_tangent: f32,
    pub y_tangent: f32,
    pub x_bitangent: f32,
    pub y_bitangent: f32,
}

pub struct Animation {}

#[derive(Debug, PartialEq)]
pub struct Goal {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub goal_type: GoalType,
}

pub struct Bumper {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub scale: Vector3,
}

pub struct Jamabar {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub scale: Vector3,
}

pub struct Banana {
    pub position: Vector3,
    pub banana_type: BananaType,
}

pub struct ConeCollisionObject {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub radius_1: f32,
    pub height: f32,
    pub radius_2: f32,
}

pub struct SphereCollisionObject {
    pub position: Vector3,
    pub radius: f32,
    pub unk0x10: u32,
}

pub struct CylinderCollisionObject {
    pub position: Vector3,
    pub radius: f32,
    pub height: f32,
    pub rotation: ShortVector3,
    pub unk0x1a: u16,
}

pub struct FalloutVolume {
    pub position: Vector3,
    pub size: Vector3,
    pub rotation: ShortVector3,
    pub unk0x1e: u16,
}

pub struct CollisionHeader {
    pub center_of_rotation_position: f32,
    pub conveyor_vector: f32,

    /*pub collision_triangles: Vec<CollisionTriangle>,
    pub collision_grid_start_x: f32,
    pub collision_grid_start_z: f32,
    pub collision_grid_step_size_x: f32,
    pub collision_grid_step_size_z: f32,
    pub collision_grid_step_count_x: u32,
    pub collision_grid_step_count_z: u32,

    pub seesaw_sensitivity: f32,
    pub seesaw_friction: f32,
    pub seesaw_spring: f32,

    pub animation_loop_point: f32,
    pub animation_state_init: AnimationState,
    pub animation_type: AnimationType,
    pub animation_id: u16,

    pub unk0x9c: u32,
    pub unk0xa0: u32,
    pub unk0xb0: u32,
    pub unk0xd0: u32,
    pub unk0xa6: u16,*/
    pub goals: Vec<Goal>,
    /*
    pub bumpers: Vec<&Bumper>,
    pub jamabars: Vec<&Jamabar>,
    pub bananas: Vec<&Banana>,
    pub cone_collision_objects: Vec<&ConeCollisionObject>,
    pub sphere_collision_objects: Vec<&SphereCollisionObject>,
    pub cylinder_collision_objects: Vec<&CylinderCollisionObject>,
    pub fallout_volumes: Vec<&FalloutVolume>,*/
}

#[derive(Default)]
pub struct StageDef {
    pub magic_number_1: f32,
    pub magic_number_2: f32,

    pub start_position: Vector3,
    pub start_rotation: ShortVector3,

    pub fallout_level: f32,

    //collision_headers: Vec<CollisionHeader>,
    pub goals: Vec<Goal>,
}

impl StageDef {
    fn new(
        reader: BufReader<File>,
        game: &Game,
        endianness: &Endianness,
    ) -> Result<Self, std::io::Error> {
        todo!();
    }
}
