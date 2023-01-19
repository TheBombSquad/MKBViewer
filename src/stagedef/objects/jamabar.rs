use super::super::common::*;

const JAMABAR_SIZE: u32 = 0x20;

#[derive(EguiInspect)]
pub struct Jamabar {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub scale: Vector3,
}

impl StageDefObject for Jamabar {
    fn get_name() -> &'static str {
        "Jamabar"
    }
    fn get_description() -> &'static str {
        "A jamabar - rectangular prisms that tilt on a fixed axis depending on the stage tilt."
    }
    fn get_size() -> u32 {
        JAMABAR_SIZE
    }
}

impl Display for Jamabar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl StageDefParsable for Jamabar {
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
