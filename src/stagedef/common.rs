pub use std::fmt::Display;
use std::sync::{Arc, Mutex};

pub use egui_inspect::EguiInspect;

use super::objects::*;

#[derive(Default)]
pub struct StageDef {
    pub magic_number_1: f32,
    pub magic_number_2: f32,

    pub start_position: Vector3,
    pub start_rotation: ShortVector3,

    pub fallout_level: f32,

    pub collision_headers: Vec<CollisionHeader>,

    pub goals: Vec<GlobalStagedefObject<Goal>>,
    pub bumpers: Vec<GlobalStagedefObject<Bumper>>,
    pub jamabars: Vec<GlobalStagedefObject<Jamabar>>,
    pub bananas: Vec<GlobalStagedefObject<Banana>>,
    pub cone_collision_objects: Vec<GlobalStagedefObject<ConeCollisionObject>>,
    pub sphere_collision_objects: Vec<GlobalStagedefObject<SphereCollisionObject>>,
    pub cylinder_collision_objects: Vec<GlobalStagedefObject<CylinderCollisionObject>>,
    pub fallout_volumes: Vec<GlobalStagedefObject<FalloutVolume>>,
}

#[derive(Debug)]
pub struct GlobalStagedefObject<T> {
    pub object: Arc<Mutex<T>>,
    pub index: u32,
}

impl<T> GlobalStagedefObject<T> {
    pub fn new(object: T, index: u32) -> Self {
        Self {
            object: Arc::new(Mutex::new(object)),
            index,
        }
    }
}

impl<T> Clone for GlobalStagedefObject<T> {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            index: self.index,
        }
    }
}

impl<T: EguiInspect> EguiInspect for GlobalStagedefObject<T> {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        let guard = self.object.lock().unwrap();
        guard.inspect(label, ui);
    }
    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        let mut guard = self.object.lock().unwrap();
        guard.inspect_mut(label, ui);
    }
}

impl<T: Display> Display for GlobalStagedefObject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.object.lock().unwrap();
        guard.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for GlobalStagedefObject<T> {
    fn eq(&self, other: &Self) -> bool {
        let guard = self.object.lock().unwrap();
        let other_guard = other.object.lock().unwrap();
        guard.eq(&other_guard)
    }
}

/// Provides a method for returning the file size of an object in a [``StageDef``].
pub trait StageDefObject {
    fn get_name() -> &'static str;
    fn get_description() -> &'static str;
    fn get_size() -> u32;
}

/// 32-bit floating point 3 dimensional vector.
#[derive(Default, Debug, PartialEq, EguiInspect)]
pub struct Vector3 {
    #[inspect(slider = false)]
    pub x: f32,
    #[inspect(slider = false)]
    pub y: f32,
    #[inspect(slider = false)]
    pub z: f32,
}

impl Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.1}, {:.1}, {:.1})", self.x, self.y, self.z)
    }
}

impl From<ShortVector3> for Vector3 {
    fn from(value: ShortVector3) -> Self {
        Self {
            x: (f32::from(value.x) / 65535.0) * 360.0,
            y: (f32::from(value.y) / 65535.0) * 360.0,
            z: (f32::from(value.z) / 65535.0) * 360.0,
        }
    }
}

/// 16-bit 'short' 3 dimensional vector. Used to represent rotations in Monkey Ball stagedefs.
#[derive(Default, Debug, PartialEq, EguiInspect, Clone, Copy)]
pub struct ShortVector3 {
    #[inspect(slider, min = 0.0, max = 65535.0)]
    pub x: u16,
    #[inspect(slider, min = 0.0, max = 65535.0)]
    pub y: u16,
    #[inspect(slider, min = 0.0, max = 65535.0)]
    pub z: u16,
}

impl Display for ShortVector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec_degrees = Vector3::from(*self);
        write!(f, "({:.1}º, {:.1}º, {:.1}º)", vec_degrees.x, vec_degrees.y, vec_degrees.z)
    }
}

#[derive(Clone, Copy)]
pub enum Game {
    SMB1,
    SMB2,
    SMBDX,
}

impl Default for Game {
    fn default() -> Self {
        Game::SMB2
    }
}

#[derive(Default)]
pub enum Endianness {
    #[default]
    BigEndian,
    LittleEndian,
}

