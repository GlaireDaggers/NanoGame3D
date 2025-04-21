use std::{collections::HashMap, fmt::Debug, str::FromStr};

use crate::{math::{Vector2, Vector3, Vector4}, misc::Color32};

pub fn parse_vec2(src: &str) -> Result<Vector2,()> {
    let split = src.split_whitespace().collect::<Vec<_>>();
    if split.len() != 2 {
        return Err(());
    }

    let x = match split[0].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let y = match split[1].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    return Ok(Vector2::new(x, y));
}

pub fn parse_vec3(src: &str) -> Result<Vector3,()> {
    let split = src.split_whitespace().collect::<Vec<_>>();
    if split.len() != 3 {
        return Err(());
    }

    let x = match split[0].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let y = match split[1].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let z = match split[2].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    return Ok(Vector3::new(x, y, z));
}

pub fn parse_vec4(src: &str) -> Result<Vector4,()> {
    let split = src.split_whitespace().collect::<Vec<_>>();
    if split.len() != 4 {
        return Err(());
    }

    let x = match split[0].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let y = match split[1].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let z = match split[2].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let w = match split[3].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    return Ok(Vector4::new(x, y, z, w));
}

pub fn parse_color32(src: &str) -> Result<Color32,()> {
    let split = src.split_whitespace().collect::<Vec<_>>();
    if split.len() != 4 {
        return Err(());
    }

    let r = match split[0].parse::<u8>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let g = match split[1].parse::<u8>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let b = match split[2].parse::<u8>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    let a = match split[3].parse::<u8>() {
        Ok(v) => v,
        Err(_) => return Err(())
    };

    return Ok(Color32::new(r, g, b, a));
}

pub fn parse_prop<T: FromStr>(props: &HashMap<&str, &str>, prop_name: &str, default_value: T) -> T
    where <T as FromStr>::Err:Debug {
    if !props.contains_key(prop_name) {
        return default_value;
    }

    return props[prop_name].parse::<T>().unwrap();
}

pub fn parse_prop_vec3(props: &HashMap<&str, &str>, prop_name: &str, default_value: Vector3) -> Vector3 {
    if !props.contains_key(prop_name) {
        return default_value;
    }

    return parse_vec3(props[prop_name]).unwrap();
}

pub fn parse_prop_modelindex(props: &HashMap<&str, &str>, prop_name: &str, default_value: usize) -> usize {
    if !props.contains_key(prop_name) {
        return default_value;
    }

    return props[prop_name][1..].parse::<i32>().unwrap() as usize - 1;
}

pub fn get_prop_str<'a>(props: &'a HashMap<&str, &str>, prop_name: &str, default_value: &'a str) -> &'a str {
    if !props.contains_key(prop_name) {
        return default_value;
    }

    return props[prop_name];
}