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
