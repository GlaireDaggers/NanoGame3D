use gltf::animation::Interpolation;

pub enum AnimationCurveInterpolationMode {
    Step,
    Linear,
    Cubic,
}

#[derive(Default, Clone, Copy)]
pub struct AnimationCurvePoint<T> where T : Default + Clone + Copy + std::ops::Add<Output = T> + std::ops::Mul::<f32, Output = T> {
    pub time: f32,
    pub tangent_in: T,
    pub tangent_out: T,
    pub value: T,
}

pub struct AnimationCurve<T> where T : Default + Clone + Copy + std::ops::Add<Output = T> + std::ops::Mul::<f32, Output = T> {
    pub interpolation_mode: AnimationCurveInterpolationMode,
    pub curve_points: Vec<AnimationCurvePoint<T>>
}

impl<T> AnimationCurve<T> where T : Default + Clone + Copy + std::ops::Add<Output = T> + std::ops::Mul::<f32, Output = T> {
    pub fn from_gltf(mode: Interpolation, timestamps: &[f32], values: &[T]) -> AnimationCurve<T> {
        if mode == Interpolation::CubicSpline {
            // cubic spline values actually contains three values per timestamp: in-tangent, value, and out-tangent
            let curve_points: Vec<AnimationCurvePoint<T>> = values.chunks_exact(3).zip(timestamps).map(|(v, t)| {
                AnimationCurvePoint { time: *t, tangent_in: v[0], tangent_out: v[2], value: v[1] }
            }).collect();

            return AnimationCurve::new(AnimationCurveInterpolationMode::Cubic, &curve_points);
        }
        else if mode == Interpolation::Linear {
            let curve_points: Vec<AnimationCurvePoint<T>> = values.iter().zip(timestamps).map(|(v, t)| {
                AnimationCurvePoint { time: *t, tangent_in: T::default(), tangent_out: T::default(), value: *v }
            }).collect();

            return AnimationCurve::new(AnimationCurveInterpolationMode::Linear, &curve_points);
        }
        else {
            let curve_points: Vec<AnimationCurvePoint<T>> = values.iter().zip(timestamps).map(|(v, t)| {
                AnimationCurvePoint { time: *t, tangent_in: T::default(), tangent_out: T::default(), value: *v }
            }).collect();

            return AnimationCurve::new(AnimationCurveInterpolationMode::Step, &curve_points);
        }
    }

    pub fn new(interpolation_mode: AnimationCurveInterpolationMode, curve_points: &[AnimationCurvePoint<T>]) -> Self {
        AnimationCurve { interpolation_mode, curve_points: curve_points.to_vec() }
    }

    pub fn linear(start: T, end: T, start_time: f32, end_time: f32) -> Self {
        AnimationCurve {
            interpolation_mode: AnimationCurveInterpolationMode::Linear,
            curve_points: [
                AnimationCurvePoint { time: start_time, tangent_in: T::default(), tangent_out: T::default(), value: start },
                AnimationCurvePoint { time: end_time, tangent_in: T::default(), tangent_out: T::default(), value: end },
            ].to_vec()
        }
    }

    pub fn sample(self: &Self, time: f32) -> T {
        if self.curve_points.len() == 0 {
            return T::default();
        }

        if time <= self.curve_points[0].time {
            return self.curve_points[0].value;
        }

        if time >= self.curve_points[self.curve_points.len() - 1].time {
            return self.curve_points[self.curve_points.len() - 1].value;
        }

        // get left+right keyframes
        let mut lhs = &self.curve_points[0];
        let mut rhs = &self.curve_points[1];

        for i in 0..self.curve_points.len() {
            if time <= self.curve_points[i].time {
                lhs = &self.curve_points[i - 1];
                rhs = &self.curve_points[i];
                break;
            }
        }

        // interpolate
        let nt = (time - lhs.time) / (rhs.time - lhs.time);

        match self.interpolation_mode {
            AnimationCurveInterpolationMode::Step => lhs.value,
            AnimationCurveInterpolationMode::Linear => Self::lerp(lhs.value, rhs.value, nt),
            AnimationCurveInterpolationMode::Cubic => Self::lerp_cubic(lhs.value, rhs.value, lhs.tangent_out, rhs.tangent_in, nt)
        }
    }

    pub fn duration(self: &Self) -> f32 {
        return self.curve_points[self.curve_points.len() - 1].time;
    }

    fn lerp(lhs: T, rhs: T, time: f32) -> T {
        return (lhs * (1.0 - time)) + (rhs * time);
    }

    fn lerp_cubic(lhs: T, rhs: T, in_tangent: T, out_tangent: T, time: f32) -> T {
        let squared = time * time;
        let cubed = time * squared;

        let part2 = (3.0 * squared) - (2.0 * cubed);
        let part1 = 1.0 - part2;
        let part4 = cubed - squared;
        let part3 = part4 - squared + time;

        return (lhs * part1) + (rhs * part2) + (in_tangent * part3) + (out_tangent * part4);
    }
}