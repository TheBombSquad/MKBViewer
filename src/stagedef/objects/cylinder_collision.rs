use super::super::common::*;

const CYL_COL_SIZE: u32 = 0x1C;

#[derive(EguiInspect)]
pub struct CylinderCollisionObject {
    pub position: Vector3,
    pub radius: f32,
    pub height: f32,
    pub rotation: ShortVector3,
    pub unk0x1a: u16,
}

impl StageDefObject for CylinderCollisionObject {
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

impl Display for CylinderCollisionObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}
