use std::sync::Arc;

use crate::graphics::model::Model;

pub struct RenderMesh {
    pub mesh: Arc<Model>
}

impl RenderMesh {
    pub fn new(mesh: Arc<Model>) -> RenderMesh {
        RenderMesh { mesh }
    }
}