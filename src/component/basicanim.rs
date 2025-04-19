pub enum AnimationLoopMode {
    Wrap,
    Clamp
}

pub struct BasicAnim {
    pub animation_id: usize,
    pub animation_time: f32,
    pub loop_mode: AnimationLoopMode,
}

impl BasicAnim {
    pub fn new(animation_id: usize, loop_mode: AnimationLoopMode) -> BasicAnim {
        BasicAnim { animation_id, animation_time: 0.0, loop_mode }
    }
}