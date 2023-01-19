use super::super::common::*;

const BANANA_SIZE: u32 = 0x10;

#[derive(EguiInspect)]
pub struct Banana {
    pub position: Vector3,
    pub banana_type: BananaType,
}

impl StageDefObject for Banana {
    fn get_name() -> &'static str {
        "Banana"
    }
    fn get_description() -> &'static str {
        "A banana object. Can also be a banana bunch."
    }
    fn get_size() -> u32 {
        BANANA_SIZE
    }
}

impl Display for Banana {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.banana_type, self.position)
    }
}

#[derive(PartialEq, FromPrimitive, ToPrimitive)]
pub enum BananaType {
    Single = 0x0,
    Bunch = 0x1,
}

impl Display for BananaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BananaType::Single => write!(f, "Single"),
            BananaType::Bunch => write!(f, "Bunch")
        }
    }
}

impl EguiInspect for BananaType {
    fn inspect(&self, _label: &str, _ui: &mut egui::Ui) {
        unimplemented!();
    }
    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        egui::ComboBox::from_label(label)
            .selected_text(format!("{self:}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, BananaType::Single, "Single");
                ui.selectable_value(self, BananaType::Bunch, "Bunch");
            });
    }
}
