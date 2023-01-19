use super::super::common::*;
use super::*;

const COLLISION_HEADER_SIZE: u32 = 0x49C;

#[derive(Default)]
pub struct CollisionHeader {
    pub center_of_rotation_position: Vector3,
    pub conveyor_vector: Vector3,

    /*pub collision_triangles: Vec<CollisionTriangle>,
    pub collision_grid_start_x: f32,
    pub collision_grid_start_z: f32,
    pub collision_grid_step_size_x: f32,
    pub collision_grid_step_size_z: f32,
    pub collision_grid_step_count_x: u32,
    pub collision_grid_step_count_z: u32,

    pub seesaw_sensitivity: f32,
    pub seesaw_friction: f32,
    pub seesaw_spring: f32,

    pub animation_loop_point: f32,
    pub animation_state_init: AnimationState,
    pub animation_type: AnimationType,
    pub animation_id: u16,

    pub unk0x9c: u32,
    pub unk0xa0: u32,
    pub unk0xb0: u32,
    pub unk0xd0: u32,
    pub unk0xa6: u16,*/
    pub goals: Vec<GlobalStagedefObject<Goal>>,
    pub bumpers: Vec<GlobalStagedefObject<Bumper>>,
    pub jamabars: Vec<GlobalStagedefObject<Jamabar>>,
    pub bananas: Vec<GlobalStagedefObject<Banana>>,
    pub cone_collisions: Vec<GlobalStagedefObject<ConeCollision>>,
    pub sphere_collisions: Vec<GlobalStagedefObject<SphereCollision>>,
    pub cylinder_collisions: Vec<GlobalStagedefObject<CylinderCollision>>,
    pub fallout_volumes: Vec<GlobalStagedefObject<FalloutVolume>>,
}

impl StageDefObject for CollisionHeader {
    // Collision headers refer back to global stagedef lists, so we handle this in a StageDefReader
    // instead
    fn get_name() -> &'static str {
        "Collision Header"
    }
    fn get_description() -> &'static str {
        "A collision header."
    }
    fn get_size() -> u32 {
        COLLISION_HEADER_SIZE
    }
}
