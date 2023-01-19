use super::super::common::*;

const BUMPER_SIZE: u32 = 0x20;

#[derive(EguiInspect)]
pub struct Bumper {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub scale: Vector3,
}

impl StageDefObject for Bumper {
    fn get_name() -> &'static str {
        "Bumper"
    }
    fn get_description() -> &'static str {
        "A bumper."
    }
    fn get_size() -> u32 {
        BUMPER_SIZE
    }
}

impl Display for Bumper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl StageDefParsable for Bumper {
    fn try_from_reader<R, B>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        B: ByteOrder,
        R: ReadBytesExtSmb,
    {
        let position = reader.read_vec3::<B>()?;
        let rotation = reader.read_vec3_short::<B>()?;
        reader.read_u8()?;
        let scale = reader.read_vec3::<B>()?;

        Ok(Self {
            position,
            rotation,
            scale,
        })
    }
}
