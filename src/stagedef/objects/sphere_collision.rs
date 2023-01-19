use super::super::common::*;

const SPHERE_COL_SIZE: u32 = 0x14;

#[derive(EguiInspect)]
pub struct SphereCollisionObject {
    pub position: Vector3,
    pub radius: f32,
    pub unk0x10: u32,
}

impl StageDefObject for SphereCollisionObject {
    fn get_name() -> &'static str {
        "Sphere Collision"
    }
    fn get_description() -> &'static str {
        "A spherical region that the ball can collide with. Used for efficiently calculating collision against sphere-shaped objects."
    }
    fn get_size() -> u32 {
        SPHERE_COL_SIZE
    }
}

impl Display for SphereCollisionObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}
