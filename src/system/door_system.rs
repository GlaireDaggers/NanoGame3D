use hecs::{CommandBuffer, World};

use crate::{component::{door::{Door, DoorLink, DoorOpener}, mapmodel::MapModel, transform3d::Transform3D, triggerable::TriggerState}, MapData, TimeData};

const DOOR_OPEN_RADIUS: f32 = 150.0;

// first pass: update Triggerable state of auto-open doors in player proximity
fn door_system_pass1(map: &MapData, world: &mut World) {
     // gather doors
     let mut door_iter = world.query::<(&Door, &mut TriggerState, &MapModel, &mut Transform3D)>();
     let doors = door_iter
         .iter()
         .collect::<Vec<_>>();
 
     // gather players
     let mut player_iter = world.query::<(&DoorOpener, &Transform3D)>();
     let players = player_iter
         .iter()
         .collect::<Vec<_>>();
 
     for (_, (door, state, mapmodel, _)) in doors {
         let submodel = &map.map.submodel_lump.submodels[mapmodel.model_idx + 1];
         let door_center = (submodel.mins + submodel.maxs) * 0.5;
 
         if door.auto_open {
             state.triggered = false;
 
             for (_, (_, ent_transform)) in &players {
                 let dist = (ent_transform.position - door_center).length_sq();
                 if dist < DOOR_OPEN_RADIUS * DOOR_OPEN_RADIUS {
                    state.triggered = true;
                    break;
                 }
             }
         }
     }
}

// second pass: propagate state of linked doors to each other
fn door_system_pass2(world: &mut World) {
    let mut cmd_buf = CommandBuffer::new();
    for (_, (door, triggerable, link)) in world.query_mut::<(&Door, &TriggerState, &DoorLink)>() {
        if door.auto_open && triggerable.triggered {
            for target in &link.links {
                cmd_buf.insert_one(*target, TriggerState {
                    triggered: triggerable.triggered
                });
            }
        }
    }

    cmd_buf.run_on(world);
}

// final pass: animate triggered doors
// todo: check if new door position overlaps another entity before moving
fn door_system_pass3(time: &TimeData, world: &mut World) {
    for (_, (door, state, transform)) in world.query_mut::<(&Door, &TriggerState, &mut Transform3D)>() {
        let target_pos = if state.triggered { door.open_pos } else { door.close_pos };
        let delta = target_pos - transform.position;
        let max_delta = door.move_speed * time.delta_time;

        let delta = if delta.length_sq() > max_delta * max_delta {
            delta.normalized() * max_delta
        }
        else {
            delta
        };

        transform.position = transform.position + delta;
    }
}

/// System which opens & closes doors in proximity to entities tagged as DoorOpener
pub fn door_system_update(time: &TimeData, map: &MapData, world: &mut World) {
    door_system_pass1(map, world);
    door_system_pass2(world);
    door_system_pass3(time, world);
}