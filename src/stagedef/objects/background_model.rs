use super::super::common::*;

const BACKGROUND_MODEL_SIZE: u32 = 0x38;

#[derive(EguiInspect)]
pub struct BackgroundModel {
    unk_0x0: u32,
    model_name: String,
    unk_0x8: u32,
    position: Vector3,
    rotation: ShortVector3,
    unk_0x1e: u16,
    scale: Vector3,
    // animation header
    // animation header 2
    // effect header: should be optional..?
}

impl StageDefObject for BackgroundModel {
    fn get_name() -> &'static str {
        "BG Model"
    }
    fn get_description() -> &'static str {
        "A background model that does not tilt with the stage."
    }
    fn get_size() -> u32 {
        BACKGROUND_MODEL_SIZE
    }
}

impl Display for BackgroundModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model_name)
    }
}

impl StageDefParsable for BackgroundModel {
    fn try_from_reader<R, B>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        B: ByteOrder,
        R: ReadBytesExtSmb,
    {
        let start_offset = reader.stream_position()?;

        let unk_0x0 = reader.read_u32::<B>()?; 
        let model_name = reader.read_model_name_from_offset::<B>()?;
        let unk_0x8 = reader.read_u32::<B>()?;
        let position = reader.read_vec3::<B>()?;
        let rotation = reader.read_vec3_short::<B>()?; 
        let unk_0x1e = reader.read_u16::<B>()?; 
        let scale = reader.read_vec3::<B>()?; 
        reader.read_u32::<B>()?;
        reader.read_u32::<B>()?;
        reader.read_u32::<B>()?;
        assert!(reader.stream_position()? == start_offset + u64::from(BACKGROUND_MODEL_SIZE));

        Ok(Self {
            unk_0x0,
            model_name,
            unk_0x8,
            position,
            rotation,
            unk_0x1e,
            scale,
        })
    }
}
