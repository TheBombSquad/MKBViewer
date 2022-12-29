//! Handles representation of the Monkey Ball stagedef format
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs, io::Cursor};

use byteorder::{BigEndian, LittleEndian};

use crate::app::FileHandleWrapper;

use egui::{Id, Response, SelectableLabel, Ui};
use egui_inspect::EguiInspect;

/// Contains a StageDef, as well as extra information about the file
///
/// By default, this will be a big-endian SMB2 stagedef
#[derive(Default)]
pub struct StageDefInstance {
    pub stagedef: StageDef,
    pub game: Game,
    pub endianness: Endianness,
    pub is_active: bool,
    pub selected: Vec<Id>,
    file: FileHandleWrapper,
}

impl StageDefInstance {
    pub fn new(file: FileHandleWrapper) -> Result<Self, std::io::Error> {
        let game = Game::SMB2;
        let endianness = Endianness::BigEndian;

        let mut reader = file.get_cursor();

        //TODO: Implement endianness/game selection
        let stagedef = match endianness {
            Endianness::BigEndian => {
                StageDef::read_stagedef::<BigEndian, Cursor<Vec<u8>>>(&mut reader, &game)
            }
            Endianness::LittleEndian => {
                StageDef::read_stagedef::<LittleEndian, Cursor<Vec<u8>>>(&mut reader, &game)
            }
        }?;

        Ok(Self {
            stagedef,
            game,
            endianness,
            file,
            selected: Vec::<Id>::new(),
            is_active: true,
        })
    }

    pub fn with_endianness(mut self, endianness: Endianness) -> StageDefInstance {
        self.endianness = endianness;
        self
    }

    pub fn with_game(mut self, game: Game) -> StageDefInstance {
        self.game = game;
        self
    }

    pub fn get_filename(&self) -> String {
        self.file.file_name.clone()
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

/// 16-bit 'short' 3 dimensional vector. Used to represent rotations in Monkey Ball stagedefs.
#[derive(Default, Debug, PartialEq, EguiInspect)]
pub struct ShortVector3 {
    #[inspect(slider, min = 0.0, max = 65535.0)]
    pub x: u16,
    #[inspect(slider, min = 0.0, max = 65535.0)]
    pub y: u16,
    #[inspect(slider, min = 0.0, max = 65535.0)]
    pub z: u16,
}

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

pub enum Endianness {
    BigEndian,
    LittleEndian,
}

impl Default for Endianness {
    fn default() -> Self {
        Endianness::BigEndian
    }
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
    fn inspect(&self, label: &'static str, ui: &mut egui::Ui) {
        unimplemented!();
    }

    fn inspect_mut(&mut self, label: &'static str, ui: &mut egui::Ui) {
        egui::ComboBox::from_label(label)
            .selected_text(format!("{:?}", self))
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

pub struct CollisionHeader {
    pub center_of_rotation_position: f32,
    pub conveyor_vector: f32,

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
    pub goals: Vec<Goal>,
    /*
    pub bumpers: Vec<&Bumper>,
    pub jamabars: Vec<&Jamabar>,
    pub bananas: Vec<&Banana>,
    pub cone_collision_objects: Vec<&ConeCollisionObject>,
    pub sphere_collision_objects: Vec<&SphereCollisionObject>,
    pub cylinder_collision_objects: Vec<&CylinderCollisionObject>,
    pub fallout_volumes: Vec<&FalloutVolume>,*/
}

#[derive(Default, EguiInspect)]
pub struct StageDef {
    #[inspect(slider = false)]
    pub magic_number_1: f32,
    #[inspect(slider = false)]
    pub magic_number_2: f32,

    pub start_position: Vector3,
    pub start_rotation: ShortVector3,

    #[inspect(slider = false)]
    pub fallout_level: f32,

    //collision_headers: Vec<CollisionHeader>,
    pub goals: Vec<Goal>,
}

impl StageDef {
    fn new(
        reader: BufReader<File>,
        game: &Game,
        endianness: &Endianness,
    ) -> Result<Self, std::io::Error> {
        todo!();
    }

    fn display_tree_element<T: EguiInspect + Display>(
        field: &T,
        label: &str,
        ctx: &mut (&mut Vec<Id>, &egui::Modifiers, &mut Ui),
    ) {
        let (selected, modifiers, ui) = ctx;
        let shift_pushed = modifiers.shift;
        let ctrl_pushed = modifiers.ctrl;
        let next_id = ui.next_auto_id();
        let is_selected = selected.contains(&next_id);

        // TODO: Implement proper multi-selection when Shift is held
        if ui
            .selectable_label(is_selected, format!("{} {}", label, field))
            .clicked()
        {
            // Selecting individual elements
            if !ctrl_pushed {
                selected.clear()
            };

            selected.push(next_id);
        }
    }

    pub fn display_tree_and_inspector(&self, selected: &mut Vec<Id>, ui: &mut Ui) -> () {
        //Vec<Box<dyn EguiInspect>> {
        let modifiers = ui.ctx().input().modifiers;
        let shift_pushed = modifiers.shift;
        let ctrl_pushed = modifiers.ctrl;

        // We want to keep track of everything that is selected
        /*
        let mut add_element = |ui: &mut Ui| {
            let next_id = ui.next_auto_id();
            let is_selected = selected.contains(&next_id);
            if ui.selectable_label(is_selected, "Label").clicked() {
                // Selecting individual elements
                if !ctrl_pushed {
                    selected.clear()
                };

                selected.push(next_id);
            }
        };*/

        let response = egui::CollapsingHeader::new("Stagedef").show(ui, |ui| {
            let ctx = &mut (selected, &modifiers, ui);
            StageDef::display_tree_element(&self.magic_number_1, "Magic Number #1: ", ctx);
            StageDef::display_tree_element(&self.magic_number_2, "Magic Number #2: ", ctx);
        });
    }
}
