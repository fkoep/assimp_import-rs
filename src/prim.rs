use ffi;
use std::{mem, slice, str};
use libc::c_uint;

/// x, y
pub type Vector2 = [f32; 2];
/// x, y, z
pub type Vector3 = [f32; 3];

/// r, g, b
pub type Color3 = [f32; 3];
/// r, g, b, a
pub type Color4 = [f32; 4];
/// b, g, r, a
pub type Texel = [f32; 4];

/// [a1..a3], [b1..b3], [c1..c3]
pub type Matrix3 = [[f32; 3]; 3];
/// [a1..a4], [b1..b4], [c1..c4], [d1..d4]
pub type Matrix4 = [[f32; 4]; 4];

/// w, x, y, z
pub type Quaternion = [f32; 4];

pub fn vec2(v: ffi::aiVector2D) -> Vector2 {
    [v.x, v.y]
}
pub fn vec3(v: ffi::aiVector3D) -> Vector3 {
    [v.x, v.y, v.z]
}
pub fn col3(v: ffi::aiColor3D) -> Color3 {
    [v.r, v.g, v.b]
}
pub fn col4(v: ffi::aiColor4D) -> Color4 {
    [v.r, v.g, v.b, v.a]
}
pub fn quat(v: ffi::aiQuaternion) -> Quaternion {
    [v.w, v.x, v.y, v.z]
}
pub fn mat3(v: ffi::aiMatrix4x4) -> Matrix3 {
    [
        [v.a1, v.a2, v.a3], 
        [v.b1, v.b2, v.b3], 
        [v.c1, v.c2, v.c3],
    ]
}
pub fn mat4(v: ffi::aiMatrix4x4) -> Matrix4 {
    [
        [v.a1, v.a2, v.a3, v.a4], 
        [v.b1, v.b2, v.b3, v.b4], 
        [v.c1, v.c2, v.c3, v.c4], 
        [v.d1, v.d2, v.d3, v.d4],
    ]
}

pub fn str<'a>(s: &'a ffi::aiString) -> Option<&'a str> {
    let len = s.length as usize;
    if len == 0 {
        return None
    }
    unsafe {
        let bytes = slice::from_raw_parts(s.data.as_ptr() as *const u8, len);
        Some(str::from_utf8(bytes).unwrap())
    }
}

pub unsafe fn slice<'a, T, U>(ptr: *const T, len: c_uint) -> &'a [U] {
    assert_eq!(mem::size_of::<T>(), mem::size_of::<U>());

    let len = len as usize;
    if len == 0 || ptr.is_null() {
        return &[]
    }
    slice::from_raw_parts(ptr as *const U, len)
}
