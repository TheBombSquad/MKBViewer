//! Handles representation of the Monkey Ball stagedef format
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{fs, io::Cursor};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use num_traits::FromPrimitive;

use crate::app::FileHandleWrapper;
use crate::parser::ReadBytesExtSmb;
use crate::parser::StageDefReader;

use egui::{Id, Response, SelectableLabel, Ui};
use egui_inspect::EguiInspect;

use anyhow::Result;

type Inspectable<'a> = (&'a mut (dyn EguiInspect), String, &'static str);

/// Contains a [``StageDef``], as well as extra information about the file
///
/// By default, this will be a big-endian SMB2 stagedef
pub struct StageDefInstance {
    pub stagedef: StageDef,
    pub game: Game,
    pub endianness: Endianness,
    pub is_active: bool,
    pub ui_state: StageDefInstanceUiState,
    file: FileHandleWrapper,
}

impl StageDefInstance {
    pub fn new(file: FileHandleWrapper) -> Result<Self> {
        let game = Game::SMB2;
        let endianness = Endianness::BigEndian;

        let reader = file.get_cursor();

        //TODO: Implement endianness/game selection
        let mut sd_reader = StageDefReader::new(reader, game);

        let stagedef = match endianness {
            Endianness::BigEndian => sd_reader.read_stagedef::<BigEndian>()?,
            Endianness::LittleEndian => sd_reader.read_stagedef::<LittleEndian>()?,
        };

        Ok(Self {
            stagedef,
            game,
            endianness,
            file,
            is_active: true,
            ui_state: StageDefInstanceUiState::default(),
        })
    }

    pub fn get_filename(&self) -> String {
        self.file.file_name.clone()
    }
}

#[derive(Default)]
pub struct StageDefInstanceUiState {
    pub selected_tree_items: HashSet<Id>,
}

impl StageDefInstanceUiState {
    fn display_tree_element<'a, T: EguiInspect + ToString>(
        &mut self,
        field: &'a mut T,
        inspector_label: &'static str,
        inspector_label_index: Option<usize>,
        inspector_description: &'static str,
        inspectables: &mut Vec<Inspectable<'a>>,
        ui: &mut Ui,
    ) {
        let modifiers = ui.ctx().input().modifiers;
        let selected = &mut self.selected_tree_items;
        let shift_pushed = modifiers.shift;
        let ctrl_pushed = modifiers.ctrl;
        let modifier_pushed = shift_pushed || ctrl_pushed;
        let next_id = ui.next_auto_id();
        let is_selected = selected.contains(&next_id);

        let formatted_label = match inspector_label_index {
            Some(i) => format!("{inspector_label} {}: {}", i + 1, field.to_string()),
            None => format!("{inspector_label}: {}", field.to_string()),
        };

        // TODO: Implement proper multi-selection when Shift is held
        if ui.selectable_label(is_selected, &formatted_label).clicked() {
            // Allow selecting individual elements
            if !modifier_pushed {
                selected.clear();
            }

            if is_selected {
                selected.remove(&next_id);
            } else {
                selected.insert(next_id);
            }
        }

        if is_selected {
            inspectables.push((field, formatted_label, inspector_description));
        }
    }

    pub fn display_tree_and_inspector<'a>(
        &mut self,
        stagedef: &'a mut StageDef,
        inspectables: &mut Vec<Inspectable<'a>>,
        ui: &mut Ui,
    ) {
        egui::CollapsingHeader::new("Stagedef").show(ui, |ui| {
            self.display_tree_element(
                &mut stagedef.magic_number_1,
                "Magic Number",
                Some(0),
                "A magic number woah",
                inspectables,
                ui,
            );
            self.display_tree_element(
                &mut stagedef.magic_number_2,
                "Magic Number",
                Some(1),
                "Another magic number woah",
                inspectables,
                ui,
            );

            self.display_tree_element(
                &mut stagedef.start_position,
                "Start Position",
                None,
                "Start Position",
                inspectables,
                ui,
            );
            self.display_tree_element(
                &mut stagedef.start_rotation,
                "Start Rotation",
                None,
                "Start Rotation",
                inspectables,
                ui,
            );

            self.display_tree_stagedef_object(ui, &mut stagedef.goals, inspectables);

            egui::CollapsingHeader::new(format!("Collision Headers ({})", stagedef.collision_headers.len())).show(
                ui,
                |ui| {
                    for (col_header_idx, col_header) in stagedef.collision_headers.iter_mut().enumerate() {
                        egui::CollapsingHeader::new(format!("Collision Header #{}", col_header_idx + 1)).show(ui, |ui| {
                            self.display_tree_stagedef_object(ui, &mut col_header.goals, inspectables);
                        });
                    }
                },
            );
        });
    }

    fn display_tree_stagedef_object<'a, T: StageDefObject + EguiInspect + Display + 'a>(
        &mut self,
        ui: &mut Ui,
        objects: &'a mut Vec<GlobalStagedefObject<T>>,
        inspectables: &mut Vec<Inspectable<'a>>,
    ) {
        let header_title = format!("{}s ({})", T::get_name(), objects.len());
        egui::CollapsingHeader::new(header_title).show(ui, |ui| {
            for (index, object) in objects.iter_mut().enumerate() {
                self.display_tree_element(object, T::get_name(), Some(index), T::get_description(), inspectables, ui);
            }
        });
    }
}

// Common structures/enums

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

pub enum AnimationState {
    Play,
    Pause,
    Reverse,
    FastForward,
    FastReverse,
}

pub enum AnimationType {
    LoopingAnimation,
    PlayOnceAnimation,
    Seesaw,
}

#[derive(Default, FromPrimitive, ToPrimitive, Debug, PartialEq)]
pub enum GoalType {
    #[default]
    Blue = 0x0,
    Green = 0x1,
    Red = 0x2,
}

impl EguiInspect for GoalType {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        unimplemented!();
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        egui::ComboBox::from_label(label)
            .selected_text(format!("{self:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, GoalType::Blue, "Blue");
                ui.selectable_value(self, GoalType::Green, "Green");
                ui.selectable_value(self, GoalType::Red, "Red");
            });
    }
}

pub enum BananaType {
    Single,
    Bunch,
}

pub struct CollisionTriangle {
    pub position: Vector3,
    pub normal: Vector3,
    pub rotation: ShortVector3,
    pub delta_x2_x1: f32,
    pub delta_y2_y1: f32,
    pub delta_x3_x1: f32,
    pub delta_y3_y1: f32,
    pub x_tangent: f32,
    pub y_tangent: f32,
    pub x_bitangent: f32,
    pub y_bitangent: f32,
}

pub struct Animation {}

#[derive(Default, Debug, PartialEq, EguiInspect)]
pub struct Goal {
    #[inspect(name = "Position")]
    pub position: Vector3,
    #[inspect(name = "Rotation")]
    pub rotation: ShortVector3,
    #[inspect(name = "Goal Type")]
    pub goal_type: GoalType,
}

impl Display for Goal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

pub struct Bumper {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub scale: Vector3,
}

pub struct Jamabar {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub scale: Vector3,
}

pub struct Banana {
    pub position: Vector3,
    pub banana_type: BananaType,
}

pub struct ConeCollisionObject {
    pub position: Vector3,
    pub rotation: ShortVector3,
    pub radius_1: f32,
    pub height: f32,
    pub radius_2: f32,
}

pub struct SphereCollisionObject {
    pub position: Vector3,
    pub radius: f32,
    pub unk0x10: u32,
}

pub struct CylinderCollisionObject {
    pub position: Vector3,
    pub radius: f32,
    pub height: f32,
    pub rotation: ShortVector3,
    pub unk0x1a: u16,
}

pub struct FalloutVolume {
    pub position: Vector3,
    pub size: Vector3,
    pub rotation: ShortVector3,
    pub unk0x1e: u16,
}

#[derive(Default, Debug)]
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
    /*
    pub bumpers: Vec<&Bumper>,
    pub jamabars: Vec<&Jamabar>,
    pub bananas: Vec<&Banana>,
    pub cone_collision_objects: Vec<&ConeCollisionObject>,
    pub sphere_collision_objects: Vec<&SphereCollisionObject>,
    pub cylinder_collision_objects: Vec<&CylinderCollisionObject>,
    pub fallout_volumes: Vec<&FalloutVolume>,*/
}

#[derive(Default)]
pub struct StageDef {
    pub magic_number_1: f32,
    pub magic_number_2: f32,

    pub start_position: Vector3,
    pub start_rotation: ShortVector3,

    pub fallout_level: f32,

    pub collision_headers: Vec<CollisionHeader>,
    pub goals: Vec<GlobalStagedefObject<Goal>>,
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

const GOAL_SIZE: u32 = 0x14;
const WORMHOLE_SIZE: u32 = 0x1c;
const ALTMODEL_SIZE: u32 = 0x38;
const LEVELMODEL_PTR_A_SIZE: u32 = 0xC;
const REFLECTIVE_MODEL_SIZE: u32 = 0xC;
const LEVEL_MODEL_INSTANCE_SIZE: u32 = 0x24;
const COLLISION_TRIANGLE_SIZE: u32 = 0x40;
const FILE_HEADER_SIZE: u32 = 0x89C;
const COLLISION_HEADER_SIZE: u32 = 0x49C;
const LEVELMODEL_PTR_B_SIZE: u32 = 0x4;
const START_POS_SIZE: u32 = 0x14;
const FALLOUT_POS_SIZE: u32 = 0x4;
const FOG_ANIMATION_HEADER_SIZE: u32 = 0x30;
const FOG_HEADER_SIZE: u32 = 0x24;
const MYSTERY_3_SIZE: u32 = 0x24;
const ALT_MODEL_ANIM_HEADER_TYPE1_SIZE: u32 = 0x50;
const ALT_MODEL_ANIM_HEADER_TYPE2_SIZE: u32 = 0x60;
const EFFECT_HEADER_SIZE: u32 = 0x30;
const TEXTURE_SCROLL_SIZE: u32 = 0x8;
const LEVEL_MODEL_SIZE: u32 = 0x10;
const COLLISION_TRIANGLE_LIST_PTR_SIZE: u32 = 0x4;
const MYSTERY_5_SIZE: u32 = 0x14;
const FILE_HEADER_SIZE_SMB1: u32 = 0xA0;
const LEVELMODEL_PTR_A_SIZE_SMB1: u32 = 0xC;
const REFLECTIVE_MODEL_SIZE_SMB1: u32 = 0x8;
const LEVEL_MODEL_SIZE_SMB1: u32 = 0x4;
const ANIMATION_HEADER_SIZE: u32 = 0x40;
const ALT_ANIMATION_TYPE2_SIZE: u32 = 0x60;

/// Provides a method for returning the file size of an object in a [``StageDef``].
pub trait StageDefObject {
    fn get_size() -> u32;
    fn try_from_reader<R, B>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        B: ByteOrder,
        R: ReadBytesExtSmb + ReadBytesExt + Read;
    fn get_name() -> &'static str;
    fn get_description() -> &'static str;
}

impl StageDefObject for CollisionHeader {
    fn get_size() -> u32 {
        COLLISION_HEADER_SIZE
    }
    // Collision headers refer back to global stagedef lists, so we handle this in a StageDefReader
    // instead
    fn try_from_reader<R, B>(_reader: &mut R) -> Result<Self> {
        unimplemented!();
    }
    fn get_name() -> &'static str {
        "Collision Header"
    }
    fn get_description() -> &'static str {
        "A collision header."
    }
}

impl StageDefObject for Goal {
    fn get_size() -> u32 {
        GOAL_SIZE
    }
    fn try_from_reader<R, B>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        B: ByteOrder,
        R: ReadBytesExtSmb + ReadBytesExt + Read,
    {
        let position = reader.read_vec3::<B>()?;
        let rotation = reader.read_vec3_short::<B>()?;

        let goal_type: GoalType = FromPrimitive::from_u8(reader.read_u8()?).unwrap_or_default();
        reader.read_u8()?;

        Ok(Goal {
            position,
            rotation,
            goal_type,
        })
    }
    fn get_name() -> &'static str {
        "Goal"
    }
    fn get_description() -> &'static str {
        "A goal object. The collision for goals is hardcoded."
    }
}
