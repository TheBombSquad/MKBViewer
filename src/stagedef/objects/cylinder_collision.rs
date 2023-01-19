use super::super::common::*;

const CYL_COL_SIZE: u32 = 0x1C;

#[derive(EguiInspect)]
pub struct CylinderCollision {
    pub position: Vector3,
    pub radius: f32,
    pub height: f32,
    pub rotation: ShortVector3,
    pub unk0x1a: u16,
}

impl StageDefObject for CylinderCollision {
    fn get_name() -> &'static str {
        "Cylinder Collision"
    }
    fn get_description() -> &'static str {
        "A cylindrical region that the ball can collide with. Used for efficiently calculating collision against cylinder-shaped objects."
    }
    fn get_size() -> u32 {
        CYL_COL_SIZE
    }
}

impl Display for CylinderCollision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl StageDefParsable for CylinderCollision {
    fn try_from_reader<R, B>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        B: ByteOrder,
        R: ReadBytesExtSmb,
    {
        let position = reader.read_vec3::<B>()?;
        let radius = reader.read_f32::<B>()?;
        let height = reader.read_f32::<B>()?;
        let rotation = reader.read_vec3_short::<B>()?;
        let unk0x1a = reader.read_u16::<B>()?;

        Ok(Self {
            position,
            radius,
            height,
            rotation,
            unk0x1a,
        })
    }
}
