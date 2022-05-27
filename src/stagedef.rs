use std::{fs, collections};
use std::path::PathBuf;

pub struct StageDefInstance {
    pub stagedef: StageDef,
    pub file_path: PathBuf,
    pub game: Game,
    pub endianness: Endianness,
}

impl StageDefInstance {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let bin = match fs::read(&path) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        let file_path = path;

        //TODO: Implement
        let game = Game::SMB2;
        let endianness = Endianness::BigEndian;

        let stagedef = match StageDef::new(bin, &game, &endianness) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        Ok(Self{
            stagedef,
            file_path,
            game,
            endianness
        })
    }
}
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

pub struct ShortVector3 {
    pub x: u16,
    pub y: u16,
    pub z: u16
}

pub enum Game {
    SMB1,
    SMB2,
    SMBDX,
}

pub enum Endianness {
    BigEndian,
    LittleEndian,
}

pub struct StageDef {
    test: i32,
}

impl StageDef {
    fn new(bin: Vec<u8>, game: &Game, endianness: &Endianness) -> Result<Self, std::io::Error>{
        let test = 3;
        Ok(Self {
            test
        })
    }
}
