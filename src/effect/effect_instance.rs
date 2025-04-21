use std::sync::Arc;

use crate::{graphics::buffer::Buffer, math::{Vector2, Vector3, Vector4}, misc::Color32};

use super::effect_data::EffectData;

#[derive(Clone, Copy)]
struct ParticleVertex {
    pub _position: Vector4,
    pub _texcoord: Vector2,
    pub _color: Color32
}

#[derive(Clone, Copy)]
struct Particle {
    pub _lifetime: f32,
    pub _lifetime_delta: f32,
    pub _position: Vector3,
    pub _angle: f32,
    pub _angle_axis: Vector3,
    pub _velocity: Vector3,
    pub _angular_velocity: f32,
    pub _scale: f32,
}

enum EffectEmitterRenderer {
    None,
    Sprite {
        _vertices: Vec<ParticleVertex>,
        _indices: Vec<u16>,
        _vertex_buffer: Buffer,
        _index_buffer: Buffer,
    }
}

struct EffectEmitterInstance {
    pub _particles: Vec<Particle>,
    pub _renderer: EffectEmitterRenderer
}

pub struct EffectInstance {
    pub effect_data: Arc<EffectData>,
    emitters: Vec<EffectEmitterInstance>
}

impl EffectEmitterInstance {
    pub fn update(self: &mut EffectEmitterInstance, _delta: f32) {
    }
}

impl EffectInstance {
    pub fn new(data: &Arc<EffectData>) -> EffectInstance {
        let emitters = data.emitters.iter().map(|x| {
            let num_particles = x.emit.max_particles as usize;

            let renderer = match &x.display {
                super::effect_data::EffectDisplay::None => EffectEmitterRenderer::None,
                super::effect_data::EffectDisplay::Sprite { .. } => {
                    let num_vertices = num_particles * 4;
                    let num_indices = num_particles * 6;

                    EffectEmitterRenderer::Sprite {
                        _vertices: Vec::with_capacity(num_vertices),
                        _indices: Vec::with_capacity(num_indices),
                        _vertex_buffer: Buffer::new((num_vertices * size_of::<ParticleVertex>()) as isize),
                        _index_buffer: Buffer::new((num_indices * size_of::<u16>()) as isize),
                    }
                },
            };

            EffectEmitterInstance {
                _particles: Vec::with_capacity(num_particles),
                _renderer: renderer
            }
        }).collect::<Vec<_>>();

        EffectInstance { effect_data: data.clone(), emitters }
    }

    pub fn update(self: &mut EffectInstance, delta: f32) {
        for em in &mut self.emitters {
            em.update(delta);
        }
    }
}