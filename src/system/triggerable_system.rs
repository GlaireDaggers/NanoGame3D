use hecs::{CommandBuffer, World};

use crate::component::triggerable::{TriggerLink, TriggerState};

/// System which propagates state of triggerable entities to linked targets, if any
pub fn trigger_link_system_update(world: &mut World) {
    let mut cmd_buf = CommandBuffer::new();
    for (_, (triggerable, link)) in world.query_mut::<(&mut TriggerState, &TriggerLink)>() {
        cmd_buf.insert_one(link.target, TriggerState {
            triggered: triggerable.triggered
        });
    }

    cmd_buf.run_on(world);
}