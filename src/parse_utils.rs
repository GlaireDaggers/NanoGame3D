use std::{collections::HashMap, fmt::Debug, str::FromStr};

use crate::math::{Vector2, Vector3, Vector4};

pub fn parse_vec2(src: &str) -> Vector2 {
    let mut split = src.split_whitespace();
    let x = split.nth(0).unwrap().parse::<f32>().unwrap();
    let y = split.nth(0).unwrap().parse::<f32>().unwrap();

    return Vector2::new(x, y);
}

pub fn parse_vec3(src: &str) -> Vector3 {
    let mut split = src.split_whitespace();
    let x = split.nth(0).unwrap().parse::<f32>().unwrap();
    let y = split.nth(0).unwrap().parse::<f32>().unwrap();
    let z = split.nth(0).unwrap().parse::<f32>().unwrap();

    return Vector3::new(x, y, z);
}

pub fn parse_vec4(src: &str) -> Vector4 {
    let mut split = src.split_whitespace();
    let x = split.nth(0).unwrap().parse::<f32>().unwrap();
    let y = split.nth(0).unwrap().parse::<f32>().unwrap();
    let z = split.nth(0).unwrap().parse::<f32>().unwrap();
    let w = split.nth(0).unwrap().parse::<f32>().unwrap();

    return Vector4::new(x, y, z, w);
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

    return parse_vec3(props[prop_name]);
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