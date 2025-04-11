use crate::math::{Matrix4x4, Vector3, Vector4};

pub fn coord_space_transform() -> Matrix4x4 {
    // Quake coordinate system:
    // +X is right
    // +Y is forwards
    // +Z is up

    // GL coordinate system:
    // +X is right
    // +Y is up
    // -Z is forwards

    Matrix4x4 {m: [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, -1.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]}
}

pub fn aabb_aabb_intersects(min_a: Vector3, max_a: Vector3, min_b: Vector3, max_b: Vector3) -> bool {
    return min_a.x <= max_b.x && max_a.x >= min_b.x &&
            min_a.y <= max_b.y && max_a.y >= min_b.y &&
            min_a.z <= max_b.z && max_a.z >= min_b.z;
}

pub fn extract_frustum(viewproj: &Matrix4x4) -> [Vector4;6] {
    let row1 = Vector4::new(viewproj.m[0][0], viewproj.m[1][0], viewproj.m[2][0], viewproj.m[3][0]);
    let row2 = Vector4::new(viewproj.m[0][1], viewproj.m[1][1], viewproj.m[2][1], viewproj.m[3][1]);
    let row3 = Vector4::new(viewproj.m[0][2], viewproj.m[1][2], viewproj.m[2][2], viewproj.m[3][2]);
    let row4 = Vector4::new(viewproj.m[0][3], viewproj.m[1][3], viewproj.m[2][3], viewproj.m[3][3]);

    [
        row4 + row1,
        row4 - row1,
        row4 + row2,
        row4 - row2,
        row4 + row3,
        row4 - row3,
    ]
}

pub fn aabb_frustum(min: Vector3, max: Vector3, frustum: &[Vector4]) -> bool {
    for plane in frustum {
        if plane.dot(Vector4::new(min.x, min.y, min.z, 1.0)) <= 0.0 &&
            plane.dot(Vector4::new(max.x, min.y, min.z, 1.0)) <= 0.0 &&
            plane.dot(Vector4::new(min.x, max.y, min.z, 1.0)) <= 0.0 &&
            plane.dot(Vector4::new(max.x, max.y, min.z, 1.0)) <= 0.0 &&
            plane.dot(Vector4::new(min.x, min.y, max.z, 1.0)) <= 0.0 &&
            plane.dot(Vector4::new(max.x, min.y, max.z, 1.0)) <= 0.0 &&
            plane.dot(Vector4::new(min.x, max.y, max.z, 1.0)) <= 0.0 &&
            plane.dot(Vector4::new(max.x, max.y, max.z, 1.0)) <= 0.0 {
            return false;
        }
    }

    return true;
}

/// Transform an AABB from local space into world space, returning center + extents
pub fn transform_aabb(offset: Vector3, extents: Vector3, local2world: Matrix4x4) -> (Vector3, Vector3) {
    // get bounds corners in local space
    let corners = [
        offset + Vector3::new(-extents.x, -extents.y, -extents.z),
        offset + Vector3::new( extents.x, -extents.y, -extents.z),
        offset + Vector3::new(-extents.x,  extents.y, -extents.z),
        offset + Vector3::new( extents.x,  extents.y, -extents.z),
        offset + Vector3::new(-extents.x, -extents.y,  extents.z),
        offset + Vector3::new( extents.x, -extents.y,  extents.z),
        offset + Vector3::new(-extents.x,  extents.y,  extents.z),
        offset + Vector3::new( extents.x,  extents.y,  extents.z),
    ];

    // transform each corner to world space & get min/max extents

    let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

    for c in corners {
        let wspace_c = local2world * Vector4::new(c.x, c.y, c.z, 1.0);
        min.x = min.x.min(wspace_c.x);
        min.y = min.y.min(wspace_c.y);
        min.z = min.z.min(wspace_c.z);
        max.x = max.x.max(wspace_c.x);
        max.y = max.y.max(wspace_c.y);
        max.z = max.z.max(wspace_c.z);
    }

    ((max + min) * 0.5, (max - min) * 0.5)
}