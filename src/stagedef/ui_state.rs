use std::collections::HashSet;
use egui::{Id, Ui};
use super::common::*;

type Inspectable<'a> = (&'a mut (dyn EguiInspect), String, &'static str);

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
            self.display_tree_stagedef_object(ui, &mut stagedef.bumpers, inspectables);
            self.display_tree_stagedef_object(ui, &mut stagedef.jamabars, inspectables);
            self.display_tree_stagedef_object(ui, &mut stagedef.bananas, inspectables);
            self.display_tree_stagedef_object(ui, &mut stagedef.cone_collision_objects, inspectables);
            self.display_tree_stagedef_object(ui, &mut stagedef.sphere_collision_objects, inspectables);
            self.display_tree_stagedef_object(ui, &mut stagedef.cylinder_collision_objects, inspectables);
            self.display_tree_stagedef_object(ui, &mut stagedef.fallout_volumes, inspectables);

            egui::CollapsingHeader::new(format!("Collision Headers ({})", stagedef.collision_headers.len())).show(
                ui,
                |ui| {
                    for (col_header_idx, col_header) in stagedef.collision_headers.iter_mut().enumerate() {
                        egui::CollapsingHeader::new(format!("Collision Header #{}", col_header_idx + 1)).show(ui, |ui| {
                            self.display_tree_stagedef_object(ui, &mut col_header.goals, inspectables);
                            self.display_tree_stagedef_object(ui, &mut col_header.bumpers, inspectables);
                            self.display_tree_stagedef_object(ui, &mut col_header.jamabars, inspectables);
                            self.display_tree_stagedef_object(ui, &mut col_header.bananas, inspectables);
                            self.display_tree_stagedef_object(ui, &mut col_header.cone_collision_objects, inspectables);
                            self.display_tree_stagedef_object(ui, &mut col_header.sphere_collision_objects, inspectables);
                            self.display_tree_stagedef_object(ui, &mut col_header.cylinder_collision_objects, inspectables);
                            self.display_tree_stagedef_object(ui, &mut col_header.fallout_volumes, inspectables);
                        });
                    }
                },
            );
        });
    }

    fn display_tree_stagedef_object<'a, T>(
        &mut self,
        ui: &mut Ui,
        objects: &'a mut Vec<GlobalStagedefObject<T>>,
        inspectables: &mut Vec<Inspectable<'a>>,
    ) where
        T: StageDefObject + EguiInspect + Display + 'a,
    {
        let header_title = format!("{}s ({})", T::get_name(), objects.len());
        egui::CollapsingHeader::new(header_title).show(ui, |ui| {
            for (index, object) in objects.iter_mut().enumerate() {
                self.display_tree_element(object, T::get_name(), Some(index), T::get_description(), inspectables, ui);
            }
        });
    }
}
