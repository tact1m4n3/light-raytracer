use std::collections::HashMap;

use glam::Vec4Swizzles;

use crate::geometry::{Geometry, Material, Triangle, Vertex};

pub fn load_from_gltf(path: &str, name: &str) -> Result<Geometry, Box<dyn std::error::Error>> {
    let (document, buffers, _) = gltf::import(path)?;

    if let Some(scene) = document
        .scenes()
        .find(|scene| scene.name().is_some_and(|n| n == name))
    {
        let mut materials = vec![Material {
            albedo: glam::vec3(1.0, 0.0, 1.0),
            roughness: 0.0,
            metallic: 0.0,
            emission: glam::Vec3::ZERO,
        }];
        let mut materials_map: HashMap<usize, u32> = HashMap::new();
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut triangles: Vec<Triangle> = Vec::new();

        let transform_matrix = glam::Mat4::IDENTITY;
        for node in scene.nodes() {
            handle_node(
                &node,
                &buffers,
                &mut materials,
                &mut materials_map,
                &mut vertices,
                &mut triangles,
                transform_matrix,
            );
        }

        Ok(Geometry {
            materials,
            vertices,
            triangles,
        })
    } else {
        Err(Box::new(SceneNotFoundError))
    }
}

#[derive(Debug)]
pub struct SceneNotFoundError;

impl std::fmt::Display for SceneNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SceneNotFoundError")
    }
}

impl std::error::Error for SceneNotFoundError {}

fn iterate_children(
    nodes: gltf::scene::iter::Children,
    buffers: &[gltf::buffer::Data],
    materials: &mut Vec<Material>,
    materials_map: &mut HashMap<usize, u32>,
    vertices: &mut Vec<Vertex>,
    triangles: &mut Vec<Triangle>,
    transform_matrix: glam::Mat4,
) {
    for node in nodes {
        handle_node(
            &node,
            buffers,
            materials,
            materials_map,
            vertices,
            triangles,
            transform_matrix,
        );
    }
}

fn handle_node(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    materials: &mut Vec<Material>,
    materials_map: &mut HashMap<usize, u32>,
    vertices: &mut Vec<Vertex>,
    triangles: &mut Vec<Triangle>,
    mut transform_matrix: glam::Mat4,
) {
    transform_matrix *= glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    if let Some(gltf_mesh) = node.mesh() {
        for prim in gltf_mesh
            .primitives()
            .filter(|prim| prim.mode() == gltf::mesh::Mode::Triangles)
        {
            let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));

            let mut indices = Vec::new();
            if let Some(gltf::mesh::util::ReadIndices::U16(gltf::accessor::Iter::Standard(iter))) =
                reader.read_indices()
            {
                for v in iter {
                    indices.push(v as u32);
                }
            }

            let mut positions = Vec::new();
            if let Some(iter) = reader.read_positions() {
                for v in iter {
                    positions.push(v);
                }
            }

            let mut tex_coords = Vec::new();
            if let Some(gltf::mesh::util::ReadTexCoords::F32(gltf::accessor::Iter::Standard(
                iter,
            ))) = reader.read_tex_coords(0)
            {
                for v in iter {
                    tex_coords.push(v);
                }
            }

            let mut normals = Vec::new();
            if let Some(iter) = reader.read_normals() {
                for v in iter {
                    normals.push(v);
                }
            }

            let index_offset = vertices.len() as u32;

            for i in 0..positions.len() {
                vertices.push(Vertex {
                    position: transform_matrix.transform_point3(glam::Vec3::from(positions[i])),
                    tex_coord: tex_coords[i].into(),
                    normal: normals[i].into(),
                });
            }

            let material_index = if let Some(gltf_mat_idx) = prim.material().index() {
                *materials_map.entry(gltf_mat_idx).or_insert_with(|| {
                    let pbr_metallic_roughness = prim.material().pbr_metallic_roughness();
                    let material_index = materials.len();
                    materials.push(Material {
                        albedo: glam::Vec4::from(pbr_metallic_roughness.base_color_factor()).xyz(),
                        roughness: pbr_metallic_roughness.roughness_factor(),
                        metallic: pbr_metallic_roughness.metallic_factor(),
                        emission: prim.material().emissive_factor().into(),
                    });
                    material_index as u32
                })
            } else {
                0
            };

            let mut it = indices.iter().peekable();
            while it.peek().is_some() {
                let i0 = *it.next().unwrap();
                let i1 = *it.next().unwrap();
                let i2 = *it.next().unwrap();

                triangles.push(Triangle {
                    vertex_indices: [index_offset + i0, index_offset + i1, index_offset + i2],
                    material_index,
                });
            }
        }
    }

    iterate_children(
        node.children(),
        buffers,
        materials,
        materials_map,
        vertices,
        triangles,
        transform_matrix,
    );
}
