use super::super::common::*;

const FALLOUT_VOLUME_SIZE: u32 = 0x20;

#[derive(EguiInspect)]
pub struct FalloutVolume {
    pub position: Vector3,
    pub size: Vector3,
    pub rotation: ShortVector3,
    pub unk0x1e: u16,
}

impl StageDefObject for FalloutVolume {
    fn get_name() -> &'static str {
        "Fallout Volume"
    }
    fn get_description() -> &'static str {
        "A volume that causes a fall out when the ball is within the volume."
    }
    fn get_size() -> u32 {
        FALLOUT_VOLUME_SIZE
    }
}

impl Display for FalloutVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}
