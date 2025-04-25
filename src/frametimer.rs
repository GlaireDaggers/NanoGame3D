pub struct FrameTimer {
    frame_delta_min: f32,
    frame_delta_max: f32,
    frame_delta_avg_total: f32,
    frame_delta_total_frames: f32,
    frame_delta_timer: f32,

    pub frame_time_min: f32,
    pub frame_time_max: f32,
    pub frame_time_avg: f32,

    pub history: [f32;256],
}

impl FrameTimer {
    pub fn new() -> FrameTimer {
        FrameTimer {
            frame_delta_min: 0.0,
            frame_delta_max: 0.0,
            frame_delta_avg_total: 0.0,
            frame_delta_total_frames: 0.0,
            frame_delta_timer: 0.0,
            frame_time_min: f32::MAX,
            frame_time_max: 0.0,
            frame_time_avg: 0.0,
            history: [0.0;256]
        }
    }

    pub fn update<F>(self: &mut FrameTimer, time: f32, delta: f32, history_transform: F) where F : Fn(f32) -> f32 {
        // shift history back
        for i in 0..255 {
            self.history[i] = self.history[i + 1];
        }

        // insert new history entry
        self.history[255] = history_transform(time);

        self.frame_delta_min = self.frame_delta_min.min(time);
        self.frame_delta_max = self.frame_delta_max.max(time);
        self.frame_delta_avg_total += time;
        self.frame_delta_total_frames += 1.0;

        self.frame_delta_timer += delta;

        if self.frame_delta_timer >= 0.5 {
            self.frame_delta_timer = 0.0;

            self.frame_time_min = self.frame_delta_min;
            self.frame_time_max = self.frame_delta_max;
            self.frame_time_avg = self.frame_delta_avg_total / self.frame_delta_total_frames;

            self.frame_delta_min = f32::MAX;
            self.frame_delta_max = 0.0;
            self.frame_delta_avg_total = 0.0;
            self.frame_delta_total_frames = 0.0;
        }
    }
}