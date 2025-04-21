use std::sync::Arc;

use serde::Deserialize;

use crate::{asset_loader::{load_material, load_model, load_shader, load_texture}, graphics::{anim::{AnimationCurveInterpolationMode, AnimationCurvePoint, Color32Curve, FloatCurve, Vector2Curve, Vector3Curve}, material::Material, model::Model, shader::Shader, texture::Texture}, math::{Vector2, Vector3, Vector4}, misc::Color32, parse_utils::{parse_color32, parse_vec2, parse_vec3, parse_vec4}};

pub struct SerializedResource<T> {
    pub resource: Arc<T>
}

impl<T> Clone for SerializedResource<T> {
    fn clone(&self) -> Self {
        Self { resource: self.resource.clone() }
    }
}

impl<'de> Deserialize<'de> for SerializedResource<Texture> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let m = match load_texture(&s) {
            Ok(m) => m,
            Err(_) => {
                return Err(serde::de::Error::custom("Failed loading resource"))
            }
        };

        Ok(SerializedResource { resource: m })
    }
}

impl<'de> Deserialize<'de> for SerializedResource<Shader> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let m = match load_shader(&s) {
            Ok(m) => m,
            Err(_) => {
                return Err(serde::de::Error::custom("Failed loading resource"))
            }
        };

        Ok(SerializedResource { resource: m })
    }
}

impl<'de> Deserialize<'de> for SerializedResource<Material> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let m = match load_material(&s) {
            Ok(m) => m,
            Err(_) => {
                return Err(serde::de::Error::custom("Failed loading resource"))
            }
        };

        Ok(SerializedResource { resource: m })
    }
}

impl<'de> Deserialize<'de> for SerializedResource<Model> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let m = match load_model(&s) {
            Ok(m) => m,
            Err(_) => {
                return Err(serde::de::Error::custom("Failed loading resource"))
            }
        };

        Ok(SerializedResource { resource: m })
    }
}

impl<'de> Deserialize<'de> for Vector2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        match parse_vec2(&s) {
            Ok(v) => Ok(v),
            Err(_) => Err(serde::de::Error::custom("Failed parsing Vector2"))
        }
    }
}

impl<'de> Deserialize<'de> for Vector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        match parse_vec3(&s) {
            Ok(v) => Ok(v),
            Err(_) => Err(serde::de::Error::custom("Failed parsing Vector3"))
        }
    }
}

impl<'de> Deserialize<'de> for Vector4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        match parse_vec4(&s) {
            Ok(v) => Ok(v),
            Err(_) => Err(serde::de::Error::custom("Failed parsing Vector4"))
        }
    }
}

impl<'de> Deserialize<'de> for Color32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        match parse_color32(&s) {
            Ok(v) => Ok(v),
            Err(_) => Err(serde::de::Error::custom("Failed parsing Color32"))
        }
    }
}

impl<'de> Deserialize<'de> for FloatCurve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let points = Vec::<AnimationCurvePoint<f32>>::deserialize(deserializer)?;
        Ok(FloatCurve::new(AnimationCurveInterpolationMode::Linear, &points))
    }
}

impl<'de> Deserialize<'de> for Vector2Curve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let points = Vec::<AnimationCurvePoint<Vector2>>::deserialize(deserializer)?;
        Ok(Vector2Curve::new(AnimationCurveInterpolationMode::Linear, &points))
    }
}

impl<'de> Deserialize<'de> for Vector3Curve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let points = Vec::<AnimationCurvePoint<Vector3>>::deserialize(deserializer)?;
        Ok(Vector3Curve::new(AnimationCurveInterpolationMode::Linear, &points))
    }
}

impl<'de> Deserialize<'de> for Color32Curve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let points = Vec::<AnimationCurvePoint<Color32>>::deserialize(deserializer)?;
        Ok(Color32Curve::new(AnimationCurveInterpolationMode::Linear, &points))
    }
}