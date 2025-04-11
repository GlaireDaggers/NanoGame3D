#[derive(Clone, Copy)]
pub struct FPView {
    pub yaw: f32,
    pub pitch: f32,
    pub eye_offset: f32,
}

impl FPView {
    pub fn default() -> FPView {
        FPView {
            yaw: 0.0,
            pitch: 0.0,
            eye_offset: 0.0,
        }
    }

    pub fn new(yaw: f32, pitch: f32, eye_offset: f32) -> FPView {
        FPView {
            yaw,
            pitch,
            eye_offset,
        }
    }
}