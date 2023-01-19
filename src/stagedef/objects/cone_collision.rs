use super::super::common::*;

const CONE_COL_SIZE: u32 = 0x20;

#[derive(EguiInspect)]
pub struct ConeCollisionObject {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub radius_1: f32,
    pub height: f32,
    pub radius_2: f32,
}

impl StageDefObject for ConeCollisionObject {
    fn get_name() -> &'static str {
        "Cone Collision"
    }
    fn get_description() -> &'static str {
        "A conical region that the ball can collide with. Used for efficiently calculating collision against cone-shaped objects."
    }
    fn get_size() -> u32 {
        CONE_COL_SIZE
    }
}

impl Display for ConeCollisionObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl StageDefParsable for ConeCollisionObject {
    fn try_from_reader<R, B>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        B: ByteOrder,
        R: ReadBytesExtSmb,
    {
        let position = reader.read_vec3::<B>()?;
        let rotation = reader.read_vec3_short::<B>()?;
        reader.read_u8()?;

        let radius_1 = reader.read_f32::<B>()?;
        let height = reader.read_f32::<B>()?;
        let radius_2 = reader.read_f32::<B>()?;

        Ok(Self {
            position,
            rotation,
            radius_1,
            height,
            radius_2,
        })
    }
}

