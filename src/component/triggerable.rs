use hecs::Entity;

pub struct TriggerState {
    pub triggered: bool,
}

pub struct TriggerLink {
    pub target: Entity
}