use log::{info, error};
use std::{collections::HashMap, sync::RwLock};

use lazy_static::lazy_static;

lazy_static! {
    static ref CVARS: RwLock<HashMap<String, CVar>> = RwLock::new(HashMap::new());
}

pub enum CVarValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String)
}

pub trait SetCVar<T> {
    fn from(val: T) -> CVarValue;
}

pub trait GetCVar<T> {
    fn get(&self) -> T;
}

impl GetCVar<bool> for CVarValue {
    fn get(&self) -> bool {
        match self {
            CVarValue::Bool(v) => *v,
            CVarValue::Int(v) => *v != 0,
            CVarValue::Float(v) => *v != 0.0,
            CVarValue::String(_) => panic!("Cannot convert string to bool"),
        }
    }
}

impl From<bool> for CVarValue {
    fn from(val: bool) -> CVarValue {
        CVarValue::Bool(val)
    }
}

impl GetCVar<i32> for CVarValue {
    fn get(&self) -> i32 {
        match self {
            CVarValue::Bool(v) => *v as i32,
            CVarValue::Int(v) => *v,
            CVarValue::Float(v) => *v as i32,
            CVarValue::String(_) => panic!("Cannot convert string to int"),
        }
    }
}

impl From<i32> for CVarValue {
    fn from(val: i32) -> CVarValue {
        CVarValue::Int(val)
    }
}

impl GetCVar<f32> for CVarValue {
    fn get(&self) -> f32 {
        match self {
            CVarValue::Bool(v) => *v as i32 as f32,
            CVarValue::Int(v) => *v as f32,
            CVarValue::Float(v) => *v,
            CVarValue::String(_) => panic!("Cannot convert string to float"),
        }
    }
}

impl From<f32> for CVarValue {
    fn from(val: f32) -> CVarValue {
        CVarValue::Float(val)
    }
}

impl GetCVar<String> for CVarValue {
    fn get(&self) -> String {
        match self {
            CVarValue::Bool(v) => v.to_string(),
            CVarValue::Int(v) => v.to_string(),
            CVarValue::Float(v) => v.to_string(),
            CVarValue::String(v) => v.clone(),
        }
    }
}

impl From<String> for CVarValue {
    fn from(val: String) -> CVarValue {
        CVarValue::String(val)
    }
}

struct CVar {
    pub help: String,
    pub value: Option<CVarValue>,
    pub default: CVarValue
}

pub fn define_cvar<T>(name: &str, default_val: T, help: &str) where CVarValue : From<T> {
    let mut cvars = CVARS.write().unwrap();
    
    cvars.insert(name.to_string(), CVar {
        help: help.to_string(),
        value: None,
        default: CVarValue::from(default_val)
    });
}

pub fn print_cvars() {
    let cvars = CVARS.read().unwrap();

    for (name, val) in cvars.iter() {
        let cvar_type = match val.default {
            CVarValue::Bool(_) => "bool",
            CVarValue::Int(_) => "int",
            CVarValue::Float(_) => "float",
            CVarValue::String(_) => "string",
        };
        
        info!("{} ({}) - {}", name, cvar_type, val.help);
    }
}

pub fn set_cvar(name: &str, val: &str) {
    let mut cvars = CVARS.write().unwrap();
    let cv = cvars.get_mut(name);

    if let Some(cv) = cv {
        match cv.default {
            CVarValue::Bool(_) => {
                match val.parse::<bool>() {
                    Ok(v) => {
                        cv.value = Some(CVarValue::Bool(v));
                    }
                    Err(e) => {
                        error!("Failed setting CVAR: {}", e);
                    }
                };
            },
            CVarValue::Int(_) => {
                match val.parse::<i32>() {
                    Ok(v) => {
                        cv.value = Some(CVarValue::Int(v));
                    }
                    Err(e) => {
                        error!("Failed setting CVAR: {}", e);
                    }
                };
            },
            CVarValue::Float(_) => {
                match val.parse::<f32>() {
                    Ok(v) => {
                        cv.value = Some(CVarValue::Float(v));
                    }
                    Err(e) => {
                        error!("Failed setting CVAR: {}", e);
                    }
                };
            },
            CVarValue::String(_) => {
                cv.value = Some(CVarValue::String(val.to_string()));
            },
        }
    }
    else {
        error!("Invalid/unknown CVAR: {}", name);
    }
}

pub fn get_cvar<T>(name: &str) -> T where CVarValue : GetCVar<T> {
    let cvars = CVARS.read().unwrap();
    let cv = &cvars[name];

    if let Some(val) = &cv.value {
        return val.get();
    }
    else {
        return cv.default.get();
    }
}