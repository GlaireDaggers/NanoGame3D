use std::collections::HashSet;
use hecs::Entity;

use crate::{bsp::bspfile::MASK_SOLID, math::Vector3, misc::AABB};

use super::bspfile::BspFile;

const DIST_EPSILON: f32 = 0.01;

#[derive(Clone, Copy)]
pub struct Trace {
    pub all_solid: bool,
    pub start_solid: bool,
    pub fraction: f32,
    pub end_pos: Vector3,
    pub hit_normal: Vector3,
    pub entity: Option<Entity>
}

impl BspFile {
    pub fn trace_aabb(bounds: &AABB, start: &Vector3, end: &Vector3, box_extents: Option<&Vector3>, trace: &mut Trace) -> bool {
        let planes = [
            (Vector3::unit_x(),         bounds.center.x + bounds.extents.x),
            (Vector3::unit_x() * -1.0,  (bounds.center.x - bounds.extents.x) * -1.0),
            (Vector3::unit_y(),         bounds.center.y + bounds.extents.y),
            (Vector3::unit_y() * -1.0,  (bounds.center.y - bounds.extents.y) * -1.0),
            (Vector3::unit_z(),         bounds.center.z + bounds.extents.z),
            (Vector3::unit_z() * -1.0,  (bounds.center.z - bounds.extents.z) * -1.0),
        ];

        let mut hit_normal = Vector3::zero();
        let mut enterfrac = f32::MIN;
        let mut exitfrac = 1.0;
        let mut startout = false;
        let mut getout = false;

        for (p_normal, p_dist) in planes {
            let dist = match box_extents {
                Some(v) => {
                    let offs = Vector3::new(
                        if p_normal.x < 0.0 { v.x } else { -v.x },
                        if p_normal.y < 0.0 { v.y } else { -v.y },
                        if p_normal.z < 0.0 { v.z } else { -v.z }
                    );

                    p_dist - offs.dot(p_normal)
                }
                None => {
                    p_dist
                }
            };

            let d1 = start.dot(p_normal) - dist;
            let d2 = end.dot(p_normal) - dist;

            if d2 > 0.0 {
                getout = true;
            }

            if d1 > 0.0 {
                startout = true;
            }

            if d1 > 0.0 && d2 >= d1 {
                return false;
            }

            if d1 <= 0.0 && d2 <= 0.0 {
                continue;
            }

            if d1 > d2 {
                let f = (d1 - DIST_EPSILON) / (d1 - d2);
                if f > enterfrac {
                    enterfrac = f;
                    hit_normal = p_normal;
                }
            }
            else {
                let f = (d1 + DIST_EPSILON) / (d1 - d2);
                if f < exitfrac {
                    exitfrac = f;
                }
            }
        }

        if !startout {
            trace.start_solid = true;
            if !getout {
                trace.all_solid = true;
            }

            return false;
        }

        if enterfrac < exitfrac {
            if enterfrac > f32::MIN && enterfrac < trace.fraction {
                if enterfrac < 0.0 {
                    enterfrac = 0.0;
                }

                trace.fraction = enterfrac;
                trace.hit_normal = hit_normal;
                trace.end_pos = *start + ((*end - *start) * enterfrac);

                return true;
            }
        }

        return false;
    }

    fn trace_brush(self: &Self, brush_idx: usize, start: Vector3, end: Vector3, frac_adj: f32, box_extents: Option<Vector3>, trace: &mut Trace) {
        let brush = &self.brush_lump.brushes[brush_idx];

        if brush.num_brush_sides == 0 {
            return;
        }

        let mut hit_normal = Vector3::zero();
        let mut enterfrac = f32::MIN;
        let mut exitfrac = 1.0;
        let mut startout = false;
        let mut getout = false;

        for i in 0..brush.num_brush_sides {
            let side = &self.brush_side_lump.brush_sides[(brush.first_brush_side + i) as usize];
            let plane = &self.plane_lump.planes[side.plane as usize];

            let dist = match box_extents {
                Some(v) => {
                    let offs = Vector3::new(
                        if plane.normal.x < 0.0 { v.x } else { -v.x },
                        if plane.normal.y < 0.0 { v.y } else { -v.y },
                        if plane.normal.z < 0.0 { v.z } else { -v.z }
                    );

                    plane.distance - offs.dot(plane.normal)
                }
                None => {
                    plane.distance
                }
            };

            let d1 = start.dot(plane.normal) - dist;
            let d2 = end.dot(plane.normal) - dist;

            if d2 > 0.0 {
                getout = true;
            }

            if d1 > 0.0 {
                startout = true;
            }

            if d1 > 0.0 && d2 >= d1 {
                return;
            }

            if d1 <= 0.0 && d2 <= 0.0 {
                continue;
            }

            if d1 > d2 {
                let f = (d1 - DIST_EPSILON) / (d1 - d2);
                if f > enterfrac {
                    enterfrac = f;
                    hit_normal = plane.normal;
                }
            }
            else {
                let f = (d1 + DIST_EPSILON) / (d1 - d2);
                if f < exitfrac {
                    exitfrac = f;
                }
            }
        }
        
        if !startout {
            trace.start_solid = true;
            if !getout {
                trace.all_solid = true;
            }

            return;
        }

        if enterfrac < exitfrac {
            if enterfrac > f32::MIN && enterfrac < trace.fraction {
                if enterfrac < 0.0 {
                    enterfrac = 0.0;
                }

                trace.fraction = enterfrac + frac_adj;
                trace.hit_normal = hit_normal;
            }
        }
    }

    fn trace_leaf(self: &Self, leaf_index: usize, checked_brush: &mut HashSet<u16>, content_mask: u32, start: Vector3, end: Vector3, frac_adj: f32, box_extents: Option<Vector3>, trace: &mut Trace) {
        let leaf = &self.leaf_lump.leaves[leaf_index];

        if leaf.contents & content_mask == 0 {
            return;
        }

        // linetrace all brushes in leaf
        for i in 0..leaf.num_leaf_brushes {
            let brush_idx = self.leaf_brush_lump.brushes[(leaf.first_leaf_brush + i) as usize];
            
            // ensure we don't process the same brush more than once during a trace
            if checked_brush.contains(&brush_idx) {
                continue;
            }
            checked_brush.insert(brush_idx);

            let brush = &self.brush_lump.brushes[brush_idx as usize];

            if brush.contents & content_mask == 0 {
                return;
            }

            self.trace_brush(brush_idx as usize, start, end, frac_adj, box_extents, trace);

            if trace.fraction <= 0.0 {
                return;
            }
        }
    }

    fn recursive_trace(self: &Self, node_idx: i32, checked_brush: &mut HashSet<u16>, content_mask: u32, p1f: f32, p2f: f32, start: Vector3, end: Vector3, frac_adj: f32, box_extents: Option<Vector3>, trace: &mut Trace) {
        if trace.fraction <= p1f {
            return;
        }
        
        if node_idx < 0 {
            self.trace_leaf((-node_idx - 1) as usize, checked_brush, content_mask, start, end, frac_adj, box_extents, trace);
            return;
        }

        let node = &self.node_lump.nodes[node_idx as usize];
        let plane = &self.plane_lump.planes[node.plane as usize];

        let (t1, t2, offset) = if plane.plane_type == 0 {
            let t1 = start.x - plane.distance;
            let t2 = end.x - plane.distance;
            let offset = match box_extents {
                Some(v) => {
                    v.x
                }
                None => {
                    0.0
                }
            };

            (t1, t2, offset)
        }
        else if plane.plane_type == 1 {
            let t1 = start.y - plane.distance;
            let t2 = end.y - plane.distance;
            let offset = match box_extents {
                Some(v) => {
                    v.y
                }
                None => {
                    0.0
                }
            };

            (t1, t2, offset)
        }
        else if plane.plane_type == 2 {
            let t1 = start.z - plane.distance;
            let t2 = end.z - plane.distance;
            let offset = match box_extents {
                Some(v) => {
                    v.z
                }
                None => {
                    0.0
                }
            };

            (t1, t2, offset)
        }
        else {
            let t1 = plane.normal.dot(start) - plane.distance;
            let t2 = plane.normal.dot(end) - plane.distance;
            let offset = match box_extents {
                Some(v) => {
                    (v.x * plane.normal.x).abs() +
                    (v.y * plane.normal.y).abs() +
                    (v.z * plane.normal.z).abs()
                },
                None => {
                    0.0
                }
            };

            (t1, t2, offset)
        };

        if t1 >= offset && t2 >= offset {
            self.recursive_trace(node.front_child, checked_brush, content_mask, p1f, p2f, start, end, frac_adj, box_extents, trace);
            return;
        }

        if t1 < -offset && t2 < -offset {
            self.recursive_trace(node.back_child, checked_brush, content_mask, p1f, p2f, start, end, frac_adj, box_extents, trace);
            return;
        }

        self.recursive_trace(node.front_child, checked_brush, content_mask, p1f, p2f, start, end, frac_adj, box_extents, trace);
        self.recursive_trace(node.back_child, checked_brush, content_mask, p1f, p2f, start, end, frac_adj, box_extents, trace);
    }

    /// Checks if a given box overlaps collision shapes
    pub fn box_check(self: &Self, content_mask: u32, start: Vector3, box_extents: Vector3) -> bool {
        let head_node = self.submodel_lump.submodels[0].headnode as i32;

        let mut trace_trace = Trace {
            all_solid: false,
            start_solid: false,
            fraction: 1.0,
            end_pos: Vector3::zero(),
            hit_normal: Vector3::zero(),
            entity: None
        };

        self.recursive_trace(head_node, &mut HashSet::<u16>::new(), content_mask, 0.0, 1.0, start, start, 0.0, Some(box_extents), &mut trace_trace);

        trace_trace.start_solid
    }

    /// Sweeps a box shape against the contents of the given submodel & returns information about what was hit and where, if any
    pub fn boxtrace(self: &Self, model_index: usize, content_mask: u32, start: Vector3, end: Vector3, box_extents: Vector3) -> Trace {
        let head_node = self.submodel_lump.submodels[model_index].headnode as i32;

        let mut trace_trace = Trace {
            all_solid: false,
            start_solid: false,
            fraction: 1.0,
            end_pos: Vector3::zero(),
            hit_normal: Vector3::zero(),
            entity: None
        };

        self.recursive_trace(head_node, &mut HashSet::<u16>::new(), content_mask, 0.0, 1.0, start, end, 0.0, Some(box_extents), &mut trace_trace);

        if trace_trace.fraction == 1.0 {
            trace_trace.end_pos = end;
        }
        else {
            trace_trace.end_pos = start + ((end - start) * trace_trace.fraction);
        }

        trace_trace
    }

    /// Trace a line against the contents of the given submodel & returns information about what was hit and where, if any
    pub fn linetrace(self: &Self, model_index: usize, content_mask: u32, start: Vector3, end: Vector3) -> Trace {
        let head_node = self.submodel_lump.submodels[model_index].headnode as i32;

        let mut trace_trace = Trace {
            all_solid: false,
            start_solid: false,
            fraction: 1.0,
            end_pos: Vector3::zero(),
            hit_normal: Vector3::zero(),
            entity: None
        };

        self.recursive_trace(head_node, &mut HashSet::<u16>::new(), content_mask, 0.0, 1.0, start, end, 0.0, None, &mut trace_trace);

        if trace_trace.fraction == 1.0 {
            trace_trace.end_pos = end;
        }
        else {
            trace_trace.end_pos = start + ((end - start) * trace_trace.fraction);
        }

        trace_trace
    }

    /// Calculate the index of the leaf node which contains the given point
    pub fn calc_leaf_index(self: &Self, position: &Vector3) -> i32 {
        let mut cur_node: i32 = 0;

        while cur_node >= 0 {
            let node = &self.node_lump.nodes[cur_node as usize];
            let plane = &self.plane_lump.planes[node.plane as usize];

            // what side of the plane is this point on
            let side = position.dot(plane.normal) - plane.distance;
            if side >= 0.0 {
                cur_node = node.front_child;
            }
            else {
                cur_node = node.back_child;
            }
        }

        // leaf indices are encoded as negative numbers: -(leaf_idx + 1)
        return -cur_node - 1;
    }

    /// Attempts to sweep a box through the world, sliding along any surfaces it hits and returning a new position and velocity as well as trace hit information
    /// 
    /// # Arguments
    /// 
    /// * 'start_pos' - The current center point of the box shape
    /// * 'velocity' - The velocity of the box shape
    /// * 'delta' - The timestep of the movement (final sweep length is velocity times delta)
    /// * 'allow_sliding' - Whether or not to allow sliding against hit surfaces
    /// * 'box_extents' - The extents of the box on each axis (half the box's total size)
    pub fn trace_move<TraceFn>(self: &Self, start_pos: &Vector3, velocity: &Vector3, delta: f32, allow_sliding: bool, box_extents: Vector3, trace_fn: TraceFn) -> (Vector3, Vector3, Trace)
        where TraceFn: Fn(u32, &Vector3, &Vector3, &Vector3) -> Trace {
        const NUM_ITERATIONS: usize = 8;

        let mut cur_pos = *start_pos;
        let mut cur_velocity = *velocity;
        let mut remaining_delta = delta;

        let mut planes: [Vector3; NUM_ITERATIONS] = [Vector3::zero(); NUM_ITERATIONS];
        let mut num_planes: usize = 0;

        let mut ret_trace = Trace {
            all_solid: false,
            start_solid: false,
            fraction: 1.0,
            end_pos: Vector3::zero(),
            hit_normal: Vector3::zero(),
            entity: None
        };

        for _iter in 0..NUM_ITERATIONS {
            let end = cur_pos + (cur_velocity * remaining_delta);
            let trace = trace_fn(MASK_SOLID, &cur_pos, &end, &box_extents);

            if trace.all_solid {
                cur_velocity.z = 0.0; // don't build vertical velocity
                return (cur_pos, cur_velocity, trace);
            }

            if trace.fraction > 0.0 {
                num_planes = 0;
                cur_pos = trace.end_pos;
                remaining_delta -= remaining_delta * trace.fraction;
            }
            
            if ret_trace.fraction == 1.0 {
                ret_trace = trace;
            }

            if trace.fraction == 1.0 || !allow_sliding {
                break;
            }

            planes[num_planes] = trace.hit_normal;
            num_planes += 1;

            let mut broke_i: bool = false;
            for i in 0..num_planes {
                // clip velocity to plane
                let backoff = cur_velocity.dot(planes[i]) * 1.01;
                cur_velocity = cur_velocity - (planes[i] * backoff);

                let mut broke_j = false;
                for j in 0..num_planes {
                    if j != i {
                        if cur_velocity.dot(planes[j]) < 0.0 {
                            broke_j = true;
                            break;
                        }
                    }
                }

                if !broke_j {
                    broke_i = true;
                    break;
                }
            }

            if broke_i {
                // go along this plane
            }
            else {
                // go along the crease
                if num_planes != 2 {
                    break;
                }

                let dir = planes[0].cross(planes[1]);
                let d = dir.dot(cur_velocity);
                cur_velocity = dir * d;
            }
        }

        (cur_pos, cur_velocity, ret_trace)
    }
}