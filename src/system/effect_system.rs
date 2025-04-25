use hecs::World;
use rand::rngs::ThreadRng;

use crate::{component::{effect::Effect, transform3d::Transform3D}, math::Matrix4x4, gamestate::TimeData};

/// System which updates particle effects
pub fn effect_system(time: &TimeData, rng: &mut ThreadRng, world: &mut World) {
    let mut effect_iter = world.query::<(&mut Effect, &mut Transform3D)>();

    for (_, (effect, transform)) in &mut effect_iter {
        if effect.world_space {
            effect.instance.transform = Matrix4x4::scale(transform.scale)
                * Matrix4x4::rotation(transform.rotation)
                * Matrix4x4::translation(transform.position);
        }
        else {
            effect.instance.transform = Matrix4x4::identity();
        }

        effect.instance.update(rng, time.delta_time);
    }
}