use std::marker::PhantomData;

use gltf::animation::Interpolation;
use serde::Deserialize;

use crate::{math::{Quaternion, Vector2, Vector3, Vector4}, misc::Color32};

/// Trait for types which can be interpolated for animation
pub trait Interpolate<T> {
    fn interpolate_linear(lhs: &T, rhs: &T, t: f32) -> T;
    fn interpolate_cubic(lhs: &T, rhs: &T, in_tangent: &T, out_tangent: &T, weights: (f32, f32, f32, f32)) -> T;
}

pub struct FloatInterpolate {
}

impl Interpolate<f32> for FloatInterpolate {
    fn interpolate_linear(lhs: &f32, rhs: &f32, t: f32) -> f32 {
        (lhs * (1.0 - t)) + (rhs * t)
    }

    fn interpolate_cubic(lhs: &f32, rhs: &f32, in_tangent: &f32, out_tangent: &f32, weights: (f32, f32, f32, f32)) -> f32 {
        return (*lhs * weights.0) + (*rhs * weights.1) + (*in_tangent * weights.2) + (*out_tangent * weights.3);
    }
}

impl Interpolate<Vector2> for Vector2 {
    fn interpolate_linear(lhs: &Vector2, rhs: &Vector2, t: f32) -> Vector2 {
        Vector2::lerp(*lhs, *rhs, t)
    }

    fn interpolate_cubic(lhs: &Vector2, rhs: &Vector2, in_tangent: &Vector2, out_tangent: &Vector2, weights: (f32, f32, f32, f32)) -> Vector2 {
        return (*lhs * weights.0) + (*rhs * weights.1) + (*in_tangent * weights.2) + (*out_tangent * weights.3);
    }
}

impl Interpolate<Vector3> for Vector3 {
    fn interpolate_linear(lhs: &Vector3, rhs: &Vector3, t: f32) -> Vector3 {
        Vector3::lerp(*lhs, *rhs, t)
    }

    fn interpolate_cubic(lhs: &Vector3, rhs: &Vector3, in_tangent: &Vector3, out_tangent: &Vector3, weights: (f32, f32, f32, f32)) -> Vector3 {
        return (*lhs * weights.0) + (*rhs * weights.1) + (*in_tangent * weights.2) + (*out_tangent * weights.3);
    }
}

impl Interpolate<Vector4> for Vector4 {
    fn interpolate_linear(lhs: &Vector4, rhs: &Vector4, t: f32) -> Vector4 {
        Vector4::lerp(*lhs, *rhs, t)
    }

    fn interpolate_cubic(lhs: &Vector4, rhs: &Vector4, in_tangent: &Vector4, out_tangent: &Vector4, weights: (f32, f32, f32, f32)) -> Vector4 {
        return (*lhs * weights.0) + (*rhs * weights.1) + (*in_tangent * weights.2) + (*out_tangent * weights.3);
    }
}

impl Interpolate<Color32> for Color32 {
    fn interpolate_linear(lhs: &Color32, rhs: &Color32, t: f32) -> Color32 {
        Color32::from_vec4(Vector4::lerp(lhs.to_vec4(), rhs.to_vec4(), t))
    }

    fn interpolate_cubic(lhs: &Color32, rhs: &Color32, in_tangent: &Color32, out_tangent: &Color32, weights: (f32, f32, f32, f32)) -> Color32 {
        Color32::from_vec4((lhs.to_vec4() * weights.0) + (rhs.to_vec4() * weights.1) + (in_tangent.to_vec4() * weights.2) + (out_tangent.to_vec4() * weights.3))
    }
}

impl Interpolate<Quaternion> for Quaternion {
    fn interpolate_linear(lhs: &Quaternion, rhs: &Quaternion, t: f32) -> Quaternion {
        Quaternion::slerp(*lhs, *rhs, t)
    }

    fn interpolate_cubic(lhs: &Quaternion, rhs: &Quaternion, in_tangent: &Quaternion, out_tangent: &Quaternion, weights: (f32, f32, f32, f32)) -> Quaternion {
        return (*lhs * weights.0) + (*rhs * weights.1) + (*in_tangent * weights.2) + (*out_tangent * weights.3);
    }
}

pub enum AnimationCurveInterpolationMode {
    Step,
    Linear,
    Cubic,
}

#[derive(Default, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct AnimationCurvePoint<T> where T : Default + Clone + Copy {
    pub time: f32,
    pub tangent_in: T,
    pub tangent_out: T,
    pub value: T,
}

pub struct AnimationCurve<T, I> where T : Default + Clone + Copy, I : Interpolate<T> {
    pub interpolation_mode: AnimationCurveInterpolationMode,
    pub curve_points: Vec<AnimationCurvePoint<T>>,
    _phantom: PhantomData<I>
}

impl<T, I> AnimationCurve<T, I> where T : Default + Clone + Copy, I : Interpolate<T> {
    pub fn from_gltf(mode: Interpolation, timestamps: &[f32], values: &[T]) -> AnimationCurve<T, I> {
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
        AnimationCurve { interpolation_mode, curve_points: curve_points.to_vec(), _phantom: PhantomData::default() }
    }

    pub fn linear(start: T, end: T, start_time: f32, end_time: f32) -> Self {
        AnimationCurve {
            interpolation_mode: AnimationCurveInterpolationMode::Linear,
            curve_points: [
                AnimationCurvePoint { time: start_time, tangent_in: T::default(), tangent_out: T::default(), value: start },
                AnimationCurvePoint { time: end_time, tangent_in: T::default(), tangent_out: T::default(), value: end },
            ].to_vec(),
            _phantom: PhantomData::default()
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
            AnimationCurveInterpolationMode::Linear => I::interpolate_linear(&lhs.value, &rhs.value, nt),
            AnimationCurveInterpolationMode::Cubic => {
                let squared = nt * nt;
                let cubed = nt * squared;

                let part2 = (3.0 * squared) - (2.0 * cubed);
                let part1 = 1.0 - part2;
                let part4 = cubed - squared;
                let part3 = part4 - squared + nt;

                I::interpolate_cubic(&lhs.value, &rhs.value, &lhs.tangent_out, &rhs.tangent_in, (part1, part2, part3, part4))
            }
        }
    }

    pub fn duration(self: &Self) -> f32 {
        return self.curve_points[self.curve_points.len() - 1].time;
    }
}

pub type FloatCurve = AnimationCurve<f32, FloatInterpolate>;
pub type Vector2Curve = AnimationCurve<Vector2, Vector2>;
pub type Vector3Curve = AnimationCurve<Vector3, Vector3>;
pub type Vector4Curve = AnimationCurve<Vector4, Vector4>;
pub type Color32Curve = AnimationCurve<Color32, Color32>;
pub type QuaternionCurve = AnimationCurve<Quaternion, Quaternion>;