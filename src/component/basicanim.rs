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

pub struct BasicLerpAnim {
    pub animation1_id: usize,
    pub animation1_time: f32,
    pub animation2_id: usize,
    pub animation2_time: f32,
    pub mix: f32,
    pub loop_mode: AnimationLoopMode,
}

impl BasicLerpAnim {
    pub fn new(anim1_id: usize, anim2_id: usize, loop_mode: AnimationLoopMode) -> BasicLerpAnim {
        BasicLerpAnim { animation1_id: anim1_id, animation1_time: 0.0, animation2_id: anim2_id, animation2_time: 0.0, mix: 0.0, loop_mode }
    }
}