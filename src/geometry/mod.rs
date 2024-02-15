pub use for_gpu::*;

mod for_gpu;
#[cfg(feature = "gltf")]
mod gltf_loader;

#[derive(Clone, Debug)]
pub struct Geometry {
    pub materials: Vec<Material>,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            materials: vec![Material::default()],
            vertices: vec![
                Vertex {
                    position: glam::vec3(-1.0, -1.0, 0.0),
                    tex_coord: glam::vec2(0.0, 0.0),
                    normal: glam::Vec3::Z,
                },
                Vertex {
                    position: glam::vec3(1.0, -1.0, 0.0),
                    tex_coord: glam::vec2(1.0, 0.0),
                    normal: glam::Vec3::Z,
                },
                Vertex {
                    position: glam::vec3(1.0, 1.0, 0.0),
                    tex_coord: glam::vec2(1.0, 1.0),
                    normal: glam::Vec3::Z,
                },
                Vertex {
                    position: glam::vec3(-1.0, 1.0, 0.0),
                    tex_coord: glam::vec2(0.0, 1.0),
                    normal: glam::Vec3::Z,
                },
            ],
            triangles: vec![
                Triangle {
                    vertex_indices: [0, 1, 2],
                    material_index: 0,
                },
                Triangle {
                    vertex_indices: [0, 2, 3],
                    material_index: 0,
                },
            ],
        }
    }
}

impl Geometry {
    #[cfg(feature = "gltf")]
    pub fn load_from_gltf(
        path: &str,
        scene_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        gltf_loader::load_from_gltf(path, scene_name)
    }

    pub fn validate(&self) -> bool {
        for triangle in self.triangles.iter() {
            if triangle
                .vertex_indices
                .iter()
                .any(|index| *index as usize >= self.vertices.len())
                || triangle.material_index as usize >= self.materials.len()
            {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct Material {
    pub albedo: glam::Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub emission: glam::Vec3,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: glam::vec3(1.0, 0.0, 1.0),
            roughness: 1.0,
            metallic: 0.0,
            emission: glam::Vec3::ZERO,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: glam::Vec3,
    pub tex_coord: glam::Vec2,
    pub normal: glam::Vec3,
}

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertex_indices: [u32; 3],
    pub material_index: u32,
}
