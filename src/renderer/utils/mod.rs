pub use buffer::*;
pub use texture::*;

mod buffer;
mod texture;

pub fn workgroups_2d(total_size: glam::UVec2, workgroup_size: glam::UVec2) -> glam::UVec2 {
    (total_size + workgroup_size - 1) / workgroup_size
}
