
use crate::{asset_loader::EffectHandle, effect::effect_instance::EffectInstance};

pub struct Effect {
    pub instance: EffectInstance,
    pub world_space: bool
}

impl Effect {
    pub fn new(effect_data: &EffectHandle, enable_emit: bool, world_space: bool) -> Effect {
        Effect { instance: EffectInstance::new(effect_data, enable_emit), world_space }
    }
}