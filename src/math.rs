use std::ops;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const fn new(x: f32, y: f32) -> Vector2 {
        return Vector2 { x: x, y: y };
    }

    pub const fn zero() -> Vector2 {
        return Vector2 { x: 0.0, y: 0.0 };
    }

    pub const fn unit_x() -> Vector2 {
        return Vector2 { x: 1.0, y: 0.0 };
    }
    
    pub const fn unit_y() -> Vector2 {
        return Vector2 { x: 0.0, y: 1.0 };
    }

    /// Compute the squared distance between two vectors
    pub fn distance_sq(lhs: &Vector2, rhs: &Vector2) -> f32 {
        let dx = lhs.x - rhs.x;
        let dy = lhs.y - rhs.y;
        return (dx * dx) + (dy * dy);
    }

    /// Compute the distance between two vectors
    pub fn distance(lhs: &Vector2, rhs: &Vector2) -> f32 {
        let dx = lhs.x - rhs.x;
        let dy = lhs.y - rhs.y;
        return ((dx * dx) + (dy * dy)).sqrt();
    }

    /// Compute the squared length of the vector
    pub fn length_sq(self) -> f32 {
        return (self.x * self.x) + (self.y * self.y);
    }

    /// Compute the length of the vector
    pub fn length(self) -> f32 {
        return ((self.x * self.x) + (self.y * self.y)).sqrt();
    }

    /// Normalize the vector
    pub fn normalize(&mut self) {
        let mag = 1.0 / (self.x * self.x + self.y * self.y).sqrt();
        self.x *= mag;
        self.y *= mag;
    }

    /// Produce a normalized copy of the vector
    pub fn normalized(&self) -> Vector2 {
        let mag = 1.0 / (self.x * self.x + self.y * self.y).sqrt();
        return Vector2 { x: self.x * mag, y: self.y * mag };
    }

    /// Compute the dot product of two vectors
    pub fn dot(self: &Vector2, rhs: Vector2) -> f32 {
        return (self.x * rhs.x) + (self.y * rhs.y);
    }

    /// Compute linear interpolation between vectors
    pub fn lerp(v1: Self, v2: Self, t: f32) -> Self {
        (v1 * (1.0 - t)) + (v2 * t)
    }
}

impl ops::Add<Vector2> for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Vector2) -> Vector2 {
        return Vector2 { x: self.x + rhs.x, y: self.y + rhs.y };
    }
}

impl ops::Sub<Vector2> for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Vector2) -> Vector2 {
        return Vector2 { x: self.x - rhs.x, y: self.y - rhs.y };
    }
}

impl ops::Mul<Vector2> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Vector2 {
        return Vector2 { x: self.x * rhs.x, y: self.y * rhs.y };
    }
}

impl ops::Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: f32) -> Vector2 {
        return Vector2 { x: self.x * rhs, y: self.y * rhs };
    }
}

impl ops::Mul<Vector2> for f32 {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Vector2 {
        return Vector2 { x: self * rhs.x, y: self * rhs.y };
    }
}

impl ops::Div<Vector2> for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: Vector2) -> Vector2 {
        return Vector2 { x: self.x / rhs.x, y: self.y / rhs.y };
    }
}

impl ops::Div<f32> for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: f32) -> Vector2 {
        return Vector2 { x: self.x / rhs, y: self.y / rhs };
    }
}

impl ops::Div<Vector2> for f32 {
    type Output = Vector2;

    fn div(self, rhs: Vector2) -> Vector2 {
        return Vector2 { x: self / rhs.x, y: self / rhs.y };
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Vector3 {
        return Vector3 { x: x, y: y, z: z };
    }

    pub const fn zero() -> Vector3 {
        return Vector3 { x: 0.0, y: 0.0, z: 0.0 };
    }

    pub const fn unit_x() -> Vector3 {
        return Vector3 { x: 1.0, y: 0.0, z: 0.0 };
    }

    pub const fn unit_y() -> Vector3 {
        return Vector3 { x: 0.0, y: 1.0, z: 0.0 };
    }

    pub const fn unit_z() -> Vector3 {
        return Vector3 { x: 0.0, y: 0.0, z: 1.0 };
    }

    /// Compute the squared distance between two vectors
    pub fn distance_sq(lhs: &Vector3, rhs: &Vector3) -> f32 {
        let dx = lhs.x - rhs.x;
        let dy = lhs.y - rhs.y;
        let dz = lhs.z - rhs.z;
        return (dx * dx) + (dy * dy) + (dz * dz);
    }

    /// Compute the distance between two vectors
    pub fn distance(lhs: &Vector3, rhs: &Vector3) -> f32 {
        let dx = lhs.x - rhs.x;
        let dy = lhs.y - rhs.y;
        let dz = lhs.z - rhs.z;
        return ((dx * dx) + (dy * dy) + (dz * dz)).sqrt();
    }

    /// Compute the squared length of the vector
    pub fn length_sq(self) -> f32 {
        return (self.x * self.x) + (self.y * self.y) + (self.z * self.z);
    }

    /// Compute the length of the vector
    pub fn length(self) -> f32 {
        return ((self.x * self.x) + (self.y * self.y) + (self.z * self.z)).sqrt();
    }

    /// Normalize the vector
    pub fn normalize(&mut self) {
        let mag = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x *= mag;
        self.y *= mag;
        self.z *= mag;
    }

    /// Produce a normalized copy of the vector
    pub fn normalized(&self) -> Vector3 {
        let mag = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        return Vector3 { x: self.x * mag, y: self.y * mag, z: self.z * mag };
    }

    /// Compute the dot product of two vectors
    pub fn dot(self: &Vector3, rhs: Vector3) -> f32 {
        return (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z);
    }

    /// Compute the cross product of two vectors
    pub fn cross(self: &Vector3, rhs: Vector3) -> Vector3 {
        return Vector3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: -(self.x * rhs.z - self.z * rhs.x),
            z: self.x * rhs.y - self.y * rhs.x
        };
    }

    /// Compute linear interpolation between vectors
    pub fn lerp(v1: Self, v2: Self, t: f32) -> Self {
        (v1 * (1.0 - t)) + (v2 * t)
    }
}

impl ops::Add<Vector3> for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Vector3) -> Vector3 {
        return Vector3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z };
    }
}

impl ops::Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Vector3 {
        return Vector3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z };
    }
}

impl ops::Mul<Vector3> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        return Vector3 { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z };
    }
}

impl ops::Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f32) -> Vector3 {
        return Vector3 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs };
    }
}

impl ops::Mul<Vector3> for f32 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        return Vector3 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z };
    }
}

impl ops::Div<Vector3> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: Vector3) -> Vector3 {
        return Vector3 { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z };
    }
}

impl ops::Div<f32> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f32) -> Vector3 {
        return Vector3 { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs };
    }
}

impl ops::Div<Vector3> for f32 {
    type Output = Vector3;

    fn div(self, rhs: Vector3) -> Vector3 {
        return Vector3 { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z };
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
        return Vector4 { x: x, y: y, z: z, w: w };
    }

    pub const fn zero() -> Vector4 {
        return Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    }

    pub const fn unit_x() -> Vector4 {
        return Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 };
    }

    pub const fn unit_y() -> Vector4 {
        return Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 };
    }

    pub const fn unit_z() -> Vector4 {
        return Vector4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 };
    }

    pub const fn unit_w() -> Vector4 {
        return Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    }

    /// Compute the squared distance between two vectors
    pub fn distance_sq(lhs: &Vector4, rhs: &Vector4) -> f32 {
        let dx = lhs.x - rhs.x;
        let dy = lhs.y - rhs.y;
        let dz = lhs.z - rhs.z;
        let dw = lhs.w - rhs.w;
        return (dx * dx) + (dy * dy) + (dz * dz) + (dw * dw);
    }

    /// Compute the distance between two vectors
    pub fn distance(lhs: &Vector4, rhs: &Vector4) -> f32 {
        let dx = lhs.x - rhs.x;
        let dy = lhs.y - rhs.y;
        let dz = lhs.z - rhs.z;
        let dw = lhs.w - rhs.w;
        return ((dx * dx) + (dy * dy) + (dz * dz) + (dw * dw)).sqrt();
    }

    /// Compute the squared length of the vector
    pub fn length_sq(self) -> f32 {
        return (self.x * self.x) + (self.y * self.y) + (self.z * self.z) + (self.w * self.w);
    }

    /// Compute the length of the vector
    pub fn length(self) -> f32 {
        return ((self.x * self.x) + (self.y * self.y) + (self.z * self.z) + (self.w * self.w)).sqrt();
    }

    /// Normalize the vector
    pub fn normalize(&mut self) {
        let mag = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        self.x *= mag;
        self.y *= mag;
        self.z *= mag;
        self.w *= mag;
    }

    /// Produce a normalized copy of the vector
    pub fn normalized(&self) -> Vector4 {
        let mag = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        return Vector4 { x: self.x * mag, y: self.y * mag, z: self.z * mag, w: self.w * mag };
    }

    /// Compute the dot product of two vectors
    pub fn dot(self: &Vector4, rhs: Vector4) -> f32 {
        return (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w);
    }

    /// Compute linear interpolation between vectors
    pub fn lerp(v1: Self, v2: Self, t: f32) -> Self {
        (v1 * (1.0 - t)) + (v2 * t)
    }
}

impl ops::Add<Vector4> for Vector4 {
    type Output = Vector4;

    fn add(self, rhs: Vector4) -> Vector4 {
        return Vector4 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z, w: self.w + rhs.w };
    }
}

impl ops::Sub<Vector4> for Vector4 {
    type Output = Vector4;

    fn sub(self, rhs: Vector4) -> Vector4 {
        return Vector4 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z, w: self.w - rhs.w };
    }
}

impl ops::Mul<Vector4> for Vector4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Vector4 {
        return Vector4 { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z, w: self.w * rhs.w };
    }
}

impl ops::Mul<f32> for Vector4 {
    type Output = Vector4;

    fn mul(self, rhs: f32) -> Vector4 {
        return Vector4 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs, w: self.w * rhs };
    }
}

impl ops::Mul<Vector4> for f32 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Vector4 {
        return Vector4 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z, w: self * rhs.w };
    }
}

impl ops::Div<Vector4> for Vector4 {
    type Output = Vector4;

    fn div(self, rhs: Vector4) -> Vector4 {
        return Vector4 { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z, w: self.w / rhs.w };
    }
}

impl ops::Div<f32> for Vector4 {
    type Output = Vector4;

    fn div(self, rhs: f32) -> Vector4 {
        return Vector4 { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs, w: self.w / rhs };
    }
}

impl ops::Div<Vector4> for f32 {
    type Output = Vector4;

    fn div(self, rhs: Vector4) -> Vector4 {
        return Vector4 { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z, w: self / rhs.w };
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        return Quaternion { x: x, y: y, z: z, w: w };
    }

    pub const fn identity() -> Quaternion {
        return Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    }

    /// Construct a new quaternion from a rotation around the given axis
    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Quaternion {
        return Quaternion {
            x: axis.x * (angle * 0.5).sin(),
            y: axis.y * (angle * 0.5).sin(),
            z: axis.z * (angle * 0.5).sin(),
            w: (angle * 0.5).cos(),
        };
    }

    /// Construct a new quaternion from the given rotations about each axis
    pub fn from_euler(euler_angles: Vector3) -> Quaternion {
        let cx = (euler_angles.x * 0.5).cos();
        let sx = (euler_angles.x * 0.5).sin();
        let cy = (euler_angles.y * 0.5).cos();
        let sy = (euler_angles.y * 0.5).sin();
        let cz = (euler_angles.z * 0.5).cos();
        let sz = (euler_angles.z * 0.5).sin();

        return Quaternion {
            x: sx * cy * cz - cx * sy * sz,
            y: cx * sy * cz + sx * cy * sz,
            z: cx * cy * sz - sx * sy * cz,
            w: cx * cy * cz + sx * sy * sz,
        };
    }

    /// Produce a normalized copy of the quaternion
    pub fn normalized(&self) -> Quaternion {
        let mag = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        return Quaternion { x: self.x * mag, y: self.y * mag, z: self.z * mag, w: self.w * mag };
    }

    /// Produce an inverted copy of the quaternion
    pub fn inverted(&self) -> Quaternion {
        let n = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w);
        return Quaternion { x: self.x * -n, y: self.y * -n, z: self.z * -n, w: self.w * n }
    }

    pub fn slerp(q1: Quaternion, q2: Quaternion, t: f32) -> Quaternion {
        const EPSILON: f32 = 1e-6;

        let cos_omega = q1.x * q2.x + q1.y * q2.y + q1.z * q2.z + q1.w * q2.w;

        let (flip, cos_omega) = if cos_omega < 0.0 {
            (true, -cos_omega)
        }
        else {
            (false, cos_omega)
        };

        let (s1, s2) = if cos_omega > (1.0 - EPSILON) {
            (1.0 - t, if flip { -t } else { t })
        }
        else {
            let omega = cos_omega.acos();
            let inv_sin_omega = 1.0 / omega.sin();

            (
                // s1
                ((1.0 - t) * omega).sin() * inv_sin_omega,
                // s2
                if flip {
                    -(t * omega).sin() * inv_sin_omega
                }
                else {
                    (t * omega).sin() * inv_sin_omega
                }
            )
        };

        Quaternion::new(
            s1 * q1.x + s2 * q2.x,
            s1 * q1.y + s2 * q2.y,
            s1 * q1.z + s2 * q2.z,
            s1 * q1.w + s2 * q2.w
        )
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Quaternion::identity()
    }
}

impl ops::Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        let x = self.x * rhs.w + self.y * rhs.z - self.z * rhs.y + self.w * rhs.x;
        let y = -self.x * rhs.z + self.y * rhs.w + self.z * rhs.x + self.w * rhs.y;
        let z = self.x * rhs.y - self.y * rhs.x + self.z * rhs.w + self.w * rhs.z;
        let w = -self.x * rhs.x - self.y * rhs.y - self.z * rhs.z + self.w * rhs.w;
        return Quaternion { x: x, y: y, z: z, w: w };
    }
}

impl ops::Mul<Vector3> for Quaternion {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        let x = 2.0 * (self.y * rhs.z - self.z * rhs.y);
        let y = 2.0 * (self.z * rhs.x - self.x * rhs.z);
        let z = 2.0 * (self.x * rhs.y - self.y * rhs.x);

        let rx = rhs.x + x * self.w + (self.y * z - self.z * y);
        let ry = rhs.y + y * self.w + (self.z * x - self.x * z);
        let rz = rhs.z + z * self.w + (self.x * y - self.y * x);

        return Vector3 { x: rx, y: ry, z: rz };
    }
}

impl ops::Mul<f32> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: f32) -> Quaternion {
        return Quaternion { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs, w: self.w * rhs };
    }
}

impl ops::Add<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn add(self, rhs: Quaternion) -> Quaternion {
        return Quaternion { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z, w: self.w + rhs.w };
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Matrix4x4 {
    pub m: [[f32;4];4],
}

impl Matrix4x4 {
    pub const fn identity() -> Matrix4x4 {
        return Matrix4x4 { m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ] };
    }

    /// Return the given row of the matrix as a vector
    pub fn get_row(self: &Matrix4x4, index: usize) -> Vector4 {
        Vector4::new(self.m[0][index], self.m[1][index], self.m[2][index], self.m[3][index])
    }

    /// Return the given column of the matrix as a vector
    pub fn get_column(self: &Matrix4x4, index: usize) -> Vector4 {
        Vector4::new(self.m[index][0], self.m[index][1], self.m[index][2], self.m[index][3])
    }

    /// Transpose the rows and columns of the matrix
    pub fn transpose(self: &mut Matrix4x4) {
        let c0 = self.m[0];
        let c1 = self.m[1];
        let c2 = self.m[2];
        let c3 = self.m[3];

        self.m[0] = [c0[0], c1[0], c2[0], c3[0]];
        self.m[1] = [c0[1], c1[1], c2[1], c3[1]];
        self.m[2] = [c0[2], c1[2], c2[2], c3[2]];
        self.m[3] = [c0[3], c1[3], c2[3], c3[3]];
    }

    /// Returns a transposed version of the matrix
    pub fn transposed(self: &Matrix4x4) -> Matrix4x4 {
        let mut ret = *self;
        ret.transpose();

        return ret;
    }

    /// Construct a translation matrix
    pub fn translation(translation: Vector3) -> Matrix4x4 {
        return Matrix4x4 { m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [translation.x, translation.y, translation.z, 1.0],
        ] };
    }

    /// Construct a scale matrix
    pub fn scale(scale: Vector3) -> Matrix4x4 {
        return Matrix4x4 { m: [
            [scale.x, 0.0, 0.0, 0.0],
            [0.0, scale.y, 0.0, 0.0],
            [0.0, 0.0, scale.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ] };
    }

    /// Construct a rotation matrix
    pub fn rotation(rotation: Quaternion) -> Matrix4x4 {
        let num9 = rotation.x * rotation.x;
        let num8 = rotation.y * rotation.y;
        let num7 = rotation.z * rotation.z;
        let num6 = rotation.x * rotation.y;
        let num5 = rotation.z * rotation.w;
        let num4 = rotation.z * rotation.x;
        let num3 = rotation.y * rotation.w;
        let num2 = rotation.y * rotation.z;
        let num = rotation.x * rotation.w;
        
        let mut result = Matrix4x4::identity();
        result.m[0][0] = 1.0 - (2.0 * (num8 + num7));
        result.m[0][1] = 2.0 * (num6 + num5);
        result.m[0][2] = 2.0 * (num4 - num3);
        result.m[0][3] = 0.0;
        result.m[1][0] = 2.0 * (num6 - num5);
        result.m[1][1] = 1.0 - (2.0 * (num7 + num9));
        result.m[1][2] = 2.0 * (num2 + num);
        result.m[1][3] = 0.0;
        result.m[2][0] = 2.0 * (num4 + num3);
        result.m[2][1] = 2.0 * (num2 - num);
        result.m[2][2] = 1.0 - (2.0 * (num8 + num9));
        result.m[2][3] = 0.0;
        result.m[3][0] = 0.0;
        result.m[3][1] = 0.0;
        result.m[3][2] = 0.0;
        result.m[3][3] = 1.0;

        return result;
    }

    /// Construct a new off-center orthographic projection matrix
    pub fn projection_ortho(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> Matrix4x4 {
        let scale_x = 2.0 / (right - left);
        let scale_y = 2.0 / (top - bottom);
        let scale_z = 1.0 / (near - far);

        let mut mat = Matrix4x4::identity();

        mat.m[0][0] = scale_x;
        mat.m[1][1] = scale_y;
        mat.m[2][2] = scale_z;

        mat.m[3][0] = (left + right) / (left - right);
        mat.m[3][1] = (top + bottom) / (bottom - top);
        mat.m[3][2] = near / (near - far);

        return mat;
    }

    /// Construct a new orthographic projection matrix using the given aspect ratio, scale, and near/far plane clip distances
    pub fn projection_ortho_aspect(aspect_ratio: f32, scale: f32, near: f32, far: f32) -> Matrix4x4 {
        let extent_x = scale * aspect_ratio * 0.5;
        let extent_y = scale * 0.5;

        return Matrix4x4::projection_ortho(-extent_x, extent_x, extent_y, -extent_y, near, far);
    }

    /// Construct a new perspective projection matrix using the given aspect ratio, field of view, and near/far plane clip distances
    pub fn projection_perspective(aspect_ratio: f32, field_of_view: f32, near: f32, far: f32) -> Matrix4x4 {
        let top = (field_of_view * 0.5).tan() * near;
        let bottom = -top;
        let right = top * aspect_ratio;
        let left = -right;

        let height = top - bottom;
        let width = right - left;

        let two_n = 2.0 * near;

        let mut mat = Matrix4x4 {m: [
            [ 0.0, 0.0, 0.0, 0.0 ],
            [ 0.0, 0.0, 0.0, 0.0 ],
            [ 0.0, 0.0, 0.0, 0.0 ],
            [ 0.0, 0.0, 0.0, 0.0 ],
        ]};

        mat.m[0][0] = two_n / width;
        mat.m[1][1] = two_n / height;
        mat.m[2][2] = -(far + near) / (far - near);
        mat.m[2][3] = -1.0;
        mat.m[3][2] = (-2.0 * near * far) /
                    (far - near);

        return mat;
    }

    pub fn transform_point(self: Matrix4x4, point: Vector3) -> Vector3 {
        let v4 = Vector4::new(point.x, point.y, point.z, 1.0);
        let v4 = self * v4;
        Vector3::new(v4.x, v4.y, v4.z)
    }

    pub fn transform_direction(self: Matrix4x4, direction: Vector3) -> Vector3 {
        let v4 = Vector4::new(direction.x, direction.y, direction.z, 0.0);
        let v4 = self * v4;
        Vector3::new(v4.x, v4.y, v4.z).normalized()
    }
}

impl ops::Mul<Vector4> for Matrix4x4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Vector4 {
        let x = (rhs.x * self.m[0][0]) + (rhs.y * self.m[1][0]) + (rhs.z * self.m[2][0]) + (rhs.w * self.m[3][0]);
        let y = (rhs.x * self.m[0][1]) + (rhs.y * self.m[1][1]) + (rhs.z * self.m[2][1]) + (rhs.w * self.m[3][1]);
        let z = (rhs.x * self.m[0][2]) + (rhs.y * self.m[1][2]) + (rhs.z * self.m[2][2]) + (rhs.w * self.m[3][2]);
        let w = (rhs.x * self.m[0][3]) + (rhs.y * self.m[1][3]) + (rhs.z * self.m[2][3]) + (rhs.w * self.m[3][3]);

        return Vector4 { x, y, z, w };
    }
}

impl ops::Mul<f32> for Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, rhs: f32) -> Matrix4x4 {
        let m00 = self.m[0][0] * rhs;
        let m10 = self.m[1][0] * rhs;
        let m20 = self.m[2][0] * rhs;
        let m30 = self.m[3][0] * rhs;
        
        let m01 = self.m[0][1] * rhs;
        let m11 = self.m[1][1] * rhs;
        let m21 = self.m[2][1] * rhs;
        let m31 = self.m[3][1] * rhs;
        
        let m02 = self.m[0][2] * rhs;
        let m12 = self.m[1][2] * rhs;
        let m22 = self.m[2][2] * rhs;
        let m32 = self.m[3][2] * rhs;
        
        let m03 = self.m[0][3] * rhs;
        let m13 = self.m[1][3] * rhs;
        let m23 = self.m[2][3] * rhs;
        let m33 = self.m[3][3] * rhs;

        return Matrix4x4 { m: [
            [m00, m10, m20, m30],
            [m01, m11, m21, m31],
            [m02, m12, m22, m32],
            [m03, m13, m23, m33],
        ] };
    }
}

impl ops::Mul<Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, rhs: Matrix4x4) -> Matrix4x4 {
        let m00 = (self.m[0][0] * rhs.m[0][0]) + (self.m[0][1] * rhs.m[1][0]) + (self.m[0][2] * rhs.m[2][0]) + (self.m[0][3] * rhs.m[3][0]);
        let m10 = (self.m[0][0] * rhs.m[0][1]) + (self.m[0][1] * rhs.m[1][1]) + (self.m[0][2] * rhs.m[2][1]) + (self.m[0][3] * rhs.m[3][1]);
        let m20 = (self.m[0][0] * rhs.m[0][2]) + (self.m[0][1] * rhs.m[1][2]) + (self.m[0][2] * rhs.m[2][2]) + (self.m[0][3] * rhs.m[3][2]);
        let m30 = (self.m[0][0] * rhs.m[0][3]) + (self.m[0][1] * rhs.m[1][3]) + (self.m[0][2] * rhs.m[2][3]) + (self.m[0][3] * rhs.m[3][3]);

        let m01 = (self.m[1][0] * rhs.m[0][0]) + (self.m[1][1] * rhs.m[1][0]) + (self.m[1][2] * rhs.m[2][0]) + (self.m[1][3] * rhs.m[3][0]);
        let m11 = (self.m[1][0] * rhs.m[0][1]) + (self.m[1][1] * rhs.m[1][1]) + (self.m[1][2] * rhs.m[2][1]) + (self.m[1][3] * rhs.m[3][1]);
        let m21 = (self.m[1][0] * rhs.m[0][2]) + (self.m[1][1] * rhs.m[1][2]) + (self.m[1][2] * rhs.m[2][2]) + (self.m[1][3] * rhs.m[3][2]);
        let m31 = (self.m[1][0] * rhs.m[0][3]) + (self.m[1][1] * rhs.m[1][3]) + (self.m[1][2] * rhs.m[2][3]) + (self.m[1][3] * rhs.m[3][3]);

        let m02 = (self.m[2][0] * rhs.m[0][0]) + (self.m[2][1] * rhs.m[1][0]) + (self.m[2][2] * rhs.m[2][0]) + (self.m[2][3] * rhs.m[3][0]);
        let m12 = (self.m[2][0] * rhs.m[0][1]) + (self.m[2][1] * rhs.m[1][1]) + (self.m[2][2] * rhs.m[2][1]) + (self.m[2][3] * rhs.m[3][1]);
        let m22 = (self.m[2][0] * rhs.m[0][2]) + (self.m[2][1] * rhs.m[1][2]) + (self.m[2][2] * rhs.m[2][2]) + (self.m[2][3] * rhs.m[3][2]);
        let m32 = (self.m[2][0] * rhs.m[0][3]) + (self.m[2][1] * rhs.m[1][3]) + (self.m[2][2] * rhs.m[2][3]) + (self.m[2][3] * rhs.m[3][3]);

        let m03 = (self.m[3][0] * rhs.m[0][0]) + (self.m[3][1] * rhs.m[1][0]) + (self.m[3][2] * rhs.m[2][0]) + (self.m[3][3] * rhs.m[3][0]);
        let m13 = (self.m[3][0] * rhs.m[0][1]) + (self.m[3][1] * rhs.m[1][1]) + (self.m[3][2] * rhs.m[2][1]) + (self.m[3][3] * rhs.m[3][1]);
        let m23 = (self.m[3][0] * rhs.m[0][2]) + (self.m[3][1] * rhs.m[1][2]) + (self.m[3][2] * rhs.m[2][2]) + (self.m[3][3] * rhs.m[3][2]);
        let m33 = (self.m[3][0] * rhs.m[0][3]) + (self.m[3][1] * rhs.m[1][3]) + (self.m[3][2] * rhs.m[2][3]) + (self.m[3][3] * rhs.m[3][3]);

        return Matrix4x4 { m: [
            [m00, m10, m20, m30],
            [m01, m11, m21, m31],
            [m02, m12, m22, m32],
            [m03, m13, m23, m33],
        ] };
    }
}

impl ops::Add<Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;

    fn add(self, rhs: Matrix4x4) -> Matrix4x4 {
        let m00 = self.m[0][0] + rhs.m[0][0];
        let m10 = self.m[1][0] + rhs.m[1][0];
        let m20 = self.m[2][0] + rhs.m[2][0];
        let m30 = self.m[3][0] + rhs.m[3][0];
        
        let m01 = self.m[0][1] + rhs.m[0][1];
        let m11 = self.m[1][1] + rhs.m[1][1];
        let m21 = self.m[2][1] + rhs.m[2][1];
        let m31 = self.m[3][1] + rhs.m[3][1];
        
        let m02 = self.m[0][2] + rhs.m[0][2];
        let m12 = self.m[1][2] + rhs.m[1][2];
        let m22 = self.m[2][2] + rhs.m[2][2];
        let m32 = self.m[3][2] + rhs.m[3][2];
        
        let m03 = self.m[0][3] + rhs.m[0][3];
        let m13 = self.m[1][3] + rhs.m[1][3];
        let m23 = self.m[2][3] + rhs.m[2][3];
        let m33 = self.m[3][3] + rhs.m[3][3];

        return Matrix4x4 { m: [
            [m00, m10, m20, m30],
            [m01, m11, m21, m31],
            [m02, m12, m22, m32],
            [m03, m13, m23, m33],
        ] };
    }
}

impl ops::Sub<Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;

    fn sub(self, rhs: Matrix4x4) -> Matrix4x4 {
        let m00 = self.m[0][0] - rhs.m[0][0];
        let m10 = self.m[1][0] - rhs.m[1][0];
        let m20 = self.m[2][0] - rhs.m[2][0];
        let m30 = self.m[3][0] - rhs.m[3][0];
        
        let m01 = self.m[0][1] - rhs.m[0][1];
        let m11 = self.m[1][1] - rhs.m[1][1];
        let m21 = self.m[2][1] - rhs.m[2][1];
        let m31 = self.m[3][1] - rhs.m[3][1];
        
        let m02 = self.m[0][2] - rhs.m[0][2];
        let m12 = self.m[1][2] - rhs.m[1][2];
        let m22 = self.m[2][2] - rhs.m[2][2];
        let m32 = self.m[3][2] - rhs.m[3][2];
        
        let m03 = self.m[0][3] - rhs.m[0][3];
        let m13 = self.m[1][3] - rhs.m[1][3];
        let m23 = self.m[2][3] - rhs.m[2][3];
        let m33 = self.m[3][3] - rhs.m[3][3];

        return Matrix4x4 { m: [
            [m00, m10, m20, m30],
            [m01, m11, m21, m31],
            [m02, m12, m22, m32],
            [m03, m13, m23, m33],
        ] };
    }
}