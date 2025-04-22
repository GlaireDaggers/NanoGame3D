use std::{fmt::Formatter, sync::Arc};

use serde::{de::Visitor, Deserialize};

use crate::{asset_loader::{load_material, load_model, load_shader, load_texture}, graphics::{anim::{AnimationCurveInterpolationMode, AnimationCurvePoint, Color32Curve, FloatCurve, Vector2Curve, Vector3Curve}, material::Material, model::Model, shader::Shader, texture::Texture}, math::{Quaternion, Vector2, Vector3, Vector4}, misc::Color32};

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

struct Vector2Visitor;
impl<'de> Visitor<'de> for Vector2Visitor {
    type Value = Vector2;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple of numbers representing X and Y of a Vector2")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut vec: Vec<f32> = Vec::new();
        while let Some(data) = seq.next_element::<f32>()? {
            vec.push(data);
        }
        Ok(Vector2::new(vec[0], vec[1]))
    }
}

impl<'de> Deserialize<'de> for Vector2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_tuple(2, Vector2Visitor {})
    }
}

struct Vector3Visitor;
impl<'de> Visitor<'de> for Vector3Visitor {
    type Value = Vector3;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple of numbers representing X, Y, and Z of a Vector3")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut vec: Vec<f32> = Vec::new();
        while let Some(data) = seq.next_element::<f32>()? {
            vec.push(data);
        }
        Ok(Vector3::new(vec[0], vec[1], vec[2]))
    }
}

impl<'de> Deserialize<'de> for Vector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_tuple(3, Vector3Visitor {})
    }
}

struct Vector4Visitor;
impl<'de> Visitor<'de> for Vector4Visitor {
    type Value = Vector4;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple of numbers representing X, Y, Z, and W of a Vector4")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut vec: Vec<f32> = Vec::new();
        while let Some(data) = seq.next_element::<f32>()? {
            vec.push(data);
        }
        Ok(Vector4::new(vec[0], vec[1], vec[2], vec[3]))
    }
}

impl<'de> Deserialize<'de> for Vector4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_tuple(4, Vector4Visitor {})
    }
}

struct QuaternionVisitor;
impl<'de> Visitor<'de> for QuaternionVisitor {
    type Value = Quaternion;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple of numbers representing rotation in degrees around X, Y, and Z axes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut vec: Vec<f32> = Vec::new();
        while let Some(data) = seq.next_element::<f32>()? {
            vec.push(data);
        }
        let rx = vec[0].to_radians();
        let ry = vec[1].to_radians();
        let rz = vec[2].to_radians();
        Ok(Quaternion::from_euler(Vector3::new(rx, ry, rz)))
    }
}

impl<'de> Deserialize<'de> for Quaternion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_tuple(3, QuaternionVisitor {})
    }
}

struct Color32Visitor;
impl<'de> Visitor<'de> for Color32Visitor {
    type Value = Color32;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple of numbers in the 0..255 range representing R, G, B, and A channels of a 32-bit color")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut vec: Vec<u8> = Vec::new();
        while let Some(data) = seq.next_element::<u8>()? {
            vec.push(data);
        }
        Ok(Color32::new(vec[0], vec[1], vec[2], vec[3]))
    }
}

impl<'de> Deserialize<'de> for Color32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_tuple(4, Color32Visitor {})
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