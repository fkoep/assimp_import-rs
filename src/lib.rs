#[macro_use]
extern crate bitflags;
extern crate libc;

// TODO Naming? `prim`?
//pub mod types;

pub mod ffi;

#[macro_use]
mod macros;
pub mod prim;

pub mod anim;
pub mod camera;
pub mod light;
pub mod material;
pub mod mesh;
pub mod metadata;
pub mod postprocess;
pub mod texture;
pub mod scene;

// TODO config.h, importerdesc.h

pub const MAX_COLOR_SETS: usize = ffi::AI_MAX_NUMBER_OF_COLOR_SETS;
pub const MAX_TEXTURE_COORDS: usize = ffi::AI_MAX_NUMBER_OF_TEXTURECOORDS;

pub use anim::*;
pub use camera::*;
pub use material::*;
pub use light::*;
pub use mesh::*;
pub use metadata::*;
pub use postprocess::*;
pub use scene::*;
pub use texture::*;

