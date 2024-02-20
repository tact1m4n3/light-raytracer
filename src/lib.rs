/*
* TODO: implement brdf
* TODO: implement material textures
* TODO: implement bvh
* TODO: implement default skybox and the ability to not use environment maps
* TODO: better ui(live renderer)
*/

pub use camera::Camera;
pub use environment::Environment;
pub use geometry::{Geometry, Material, Triangle, Vertex};
pub use renderer::{Renderer, RendererSettings};

mod camera;
mod environment;
mod geometry;
mod renderer;
