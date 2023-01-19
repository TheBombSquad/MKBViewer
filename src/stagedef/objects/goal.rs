use super::super::common::*;

const GOAL_SIZE: u32 = 0x14;

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

impl StageDefObject for Goal {
    fn get_name() -> &'static str {
        "Goal"
    }
    fn get_description() -> &'static str {
        "A goal object. The collision for goals is hardcoded."
    }
    fn get_size() -> u32 {
        GOAL_SIZE
    }
}

#[derive(Default, FromPrimitive, ToPrimitive, Debug, PartialEq)]
pub enum GoalType {
    #[default]
    Blue = 0x0,
    Green = 0x1,
    Red = 0x2,
}

impl EguiInspect for GoalType {
    fn inspect(&self, _label: &str, _ui: &mut egui::Ui) {
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

impl StageDefParsable for Goal {
    fn try_from_reader<R, B>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        B: ByteOrder,
        R: ReadBytesExtSmb,
    {
        let position = reader.read_vec3::<B>()?;
        let rotation = reader.read_vec3_short::<B>()?;

        let goal_type: GoalType =
            FromPrimitive::from_u8(reader.read_u8()?).ok_or_else(|| anyhow::Error::msg("Failed to parse goal type"))?;
        reader.read_u8()?;

        Ok(Self {
            position,
            rotation,
            goal_type,
        })
    }
}
