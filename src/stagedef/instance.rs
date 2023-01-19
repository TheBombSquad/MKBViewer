use anyhow::Result;
use byteorder::BigEndian;
use byteorder::LittleEndian;
use crate::app::FileHandleWrapper;
use crate::parser::StageDefReader;
use super::common::*;
use super::ui_state::*;

/// Contains a [``StageDef``], as well as extra information about the file
///
/// By default, this will be a big-endian SMB2 stagedef
pub struct StageDefInstance {
    pub stagedef: StageDef,
    pub game: Game,
    pub endianness: Endianness,
    pub is_active: bool,
    pub ui_state: StageDefInstanceUiState,
    file: FileHandleWrapper,
}

impl StageDefInstance {
    pub fn new(file: FileHandleWrapper) -> Result<Self> {
        let game = Game::SMB2;
        let endianness = Endianness::BigEndian;

        let reader = file.get_cursor();

        //TODO: Implement endianness/game selection
        let mut sd_reader = StageDefReader::new(reader, game);

        let stagedef = match endianness {
            Endianness::BigEndian => sd_reader.read_stagedef::<BigEndian>()?,
            Endianness::LittleEndian => sd_reader.read_stagedef::<LittleEndian>()?,
        };

        Ok(Self {
            stagedef,
            game,
            endianness,
            file,
            is_active: true,
            ui_state: StageDefInstanceUiState::default(),
        })
    }

    pub fn get_filename(&self) -> String {
        self.file.file_name.clone()
    }
}
