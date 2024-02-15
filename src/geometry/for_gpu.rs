use super::{Material, Triangle, Vertex};

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMaterial {
    albedo: glam::Vec3,
    roughness: f32,
    metallic: f32,
    pad0: [u32; 3],
    emission: glam::Vec3,
    pad1: u32,
}

impl From<Material> for GpuMaterial {
    fn from(material: Material) -> Self {
        Self {
            albedo: material.albedo,
            roughness: material.roughness,
            metallic: material.metallic,
            pad0: [0; 3],
            emission: material.emission,
            pad1: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuVertex {
    position: glam::Vec3,
    pad0: u32,
    tex_coord: glam::Vec2,
    pad1: [u32; 2],
    normal: glam::Vec3,
    pad2: u32,
}

impl From<Vertex> for GpuVertex {
    fn from(vertex: Vertex) -> Self {
        Self {
            position: vertex.position,
            pad0: 0,
            tex_coord: vertex.tex_coord,
            pad1: [0; 2],
            normal: vertex.normal,
            pad2: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuTriangle {
    vertex_indices: [u32; 3],
    material_index: u32,
}

impl From<Triangle> for GpuTriangle {
    fn from(triangle: Triangle) -> Self {
        Self {
            vertex_indices: triangle.vertex_indices,
            material_index: triangle.material_index,
        }
    }
}
