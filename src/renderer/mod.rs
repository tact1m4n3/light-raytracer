use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    camera::{Camera, GpuCamera},
    geometry::{Geometry, GpuMaterial, GpuTriangle, GpuVertex},
};

use self::{
    passes::{BlitPass, RaytracingPass},
    utils::{StorageBuffer, Texture2D, UniformBuffer},
};

mod passes;
mod utils;

pub struct Renderer {
    size: glam::UVec2,
    max_samples: u32,
    samples_per_render: u32,
    num_samples: u32,
    pre_render_cmds: PreRenderCommands,
    acc_input_texture: Texture2D,
    acc_output_texture: Texture2D,
    output_texture: Texture2D,
    settings_uniform: UniformBuffer<SettingsUniform>,
    per_render_uniform: UniformBuffer<PerRenderUniform>,
    camera_uniform: UniformBuffer<GpuCamera>,
    materials_storage: StorageBuffer<GpuMaterial>,
    vertices_storage: StorageBuffer<GpuVertex>,
    triangles_storage: StorageBuffer<GpuTriangle>,
    raytracing_pass: RaytracingPass,
    raytracing_bind_group: wgpu::BindGroup,
    blit_pass: BlitPass,
    blit_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        size: glam::UVec2,
        output_format: wgpu::TextureFormat,
        settings: RendererSettings,
        camera: Camera,
        geometry: Geometry,
    ) -> Self {
        let acc_input_texture = Texture2D::new(
            device,
            "accumulation_input_texture",
            size,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            1,
        );

        let acc_output_texture = Texture2D::new(
            device,
            "accumulation_output_texture",
            size,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            1,
        );

        let output_texture = Texture2D::new(
            device,
            "output_texture",
            size,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            1,
        );

        let settings = if settings.validate() {
            settings
        } else {
            RendererSettings::default()
        };

        let settings_uniform = UniformBuffer::new_with_data(
            device,
            "settings_uniform",
            &[SettingsUniform {
                samples_per_render: settings.samples_per_render,
                max_ray_depth: settings.max_ray_depth,
                furnace_test: settings.furnace_test.into(),
                pad0: 0,
            }],
        );

        let per_render_uniform = UniformBuffer::new_with_data(
            device,
            "per_render_uniform",
            &[PerRenderUniform::default()],
        );

        let camera = if camera.validate() {
            camera
        } else {
            Camera::default()
        };

        let camera_uniform =
            UniformBuffer::new_with_data(device, "camera_uniform", &[GpuCamera::from(camera)]);

        let geometry = if geometry.validate() {
            geometry
        } else {
            Geometry::default()
        };

        let gpu_materials: Vec<GpuMaterial> = geometry
            .materials
            .into_iter()
            .map(GpuMaterial::from)
            .collect();
        let gpu_vertices: Vec<GpuVertex> =
            geometry.vertices.into_iter().map(GpuVertex::from).collect();
        let gpu_triangles: Vec<GpuTriangle> = geometry
            .triangles
            .into_iter()
            .map(GpuTriangle::from)
            .collect();

        let materials_storage =
            StorageBuffer::new_with_data(device, "materials_storage", &gpu_materials);
        let vertices_storage =
            StorageBuffer::new_with_data(device, "vertices_storage", &gpu_vertices);
        let triangles_storage =
            StorageBuffer::new_with_data(device, "triangles_storage", &gpu_triangles);

        let raytracing_pass = RaytracingPass::new(device);
        let raytracing_bind_group = raytracing_pass.create_bind_group(
            device,
            &acc_input_texture,
            &acc_output_texture,
            &output_texture,
            &settings_uniform,
            &per_render_uniform,
            &camera_uniform,
            &materials_storage,
            &vertices_storage,
            &triangles_storage,
        );

        let blit_pass = BlitPass::new(device, output_format);
        let blit_bind_group = blit_pass.create_bind_group(device, &output_texture);

        Self {
            size,
            max_samples: settings.max_samples,
            samples_per_render: settings.samples_per_render,
            num_samples: 0,
            pre_render_cmds: PreRenderCommands::default(),
            acc_input_texture,
            acc_output_texture,
            output_texture,
            settings_uniform,
            per_render_uniform,
            camera_uniform,
            materials_storage,
            vertices_storage,
            triangles_storage,
            raytracing_pass,
            raytracing_bind_group,
            blit_pass,
            blit_bind_group,
        }
    }

    pub fn reset(&mut self) {
        self.pre_render_cmds.reset = true;
    }

    pub fn resize(&mut self, new_size: glam::UVec2) {
        self.pre_render_cmds.reset = true;
        self.pre_render_cmds.resize = Some(new_size);
    }

    pub fn update_settings(&mut self, settings: RendererSettings) {
        self.pre_render_cmds.reset = true;
        self.pre_render_cmds.update_settings = Some(settings);
    }

    pub fn update_camera(&mut self, camera: Camera) {
        self.pre_render_cmds.reset = true;
        self.pre_render_cmds.update_camera = Some(camera);
    }

    pub fn update_geometry(&mut self, geometry: Geometry) {
        self.pre_render_cmds.reset = true;
        self.pre_render_cmds.update_geometry = Some(geometry);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut update_bind_groups = false;

        if self.pre_render_cmds.reset {
            self.num_samples = 0;
            self.pre_render_cmds.reset = false;

            encoder.clear_texture(
                self.acc_input_texture.inner(),
                &wgpu::ImageSubresourceRange {
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                },
            );
        }

        if let Some(new_size) = self.pre_render_cmds.resize.take() {
            self.size = new_size.max(glam::UVec2::ONE);

            self.acc_input_texture = Texture2D::new(
                device,
                "accumulation_input_texture",
                self.size,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                1,
            );

            self.acc_output_texture = Texture2D::new(
                device,
                "accumulation_output_texture",
                self.size,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                1,
            );

            self.output_texture = Texture2D::new(
                device,
                "output_texture",
                self.size,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                1,
            );

            update_bind_groups = true;
        }

        if let Some(settings) = self.pre_render_cmds.update_settings.take() {
            let settings = if settings.validate() {
                settings
            } else {
                RendererSettings::default()
            };

            self.max_samples = settings.max_samples;
            self.samples_per_render = settings.samples_per_render;

            self.settings_uniform.write(
                queue,
                &[SettingsUniform {
                    samples_per_render: settings.samples_per_render,
                    max_ray_depth: settings.max_ray_depth,
                    furnace_test: settings.furnace_test.into(),
                    pad0: 0,
                }],
            );
        }

        if let Some(camera) = self.pre_render_cmds.update_camera.take() {
            let camera = if camera.validate() {
                camera
            } else {
                Camera::default()
            };

            self.camera_uniform.write(queue, &[GpuCamera::from(camera)]);
        }

        if let Some(geometry) = self.pre_render_cmds.update_geometry.take() {
            let geometry = if geometry.validate() {
                geometry
            } else {
                Geometry::default()
            };

            let gpu_materials: Vec<GpuMaterial> = geometry
                .materials
                .into_iter()
                .map(GpuMaterial::from)
                .collect();
            let gpu_vertices: Vec<GpuVertex> =
                geometry.vertices.into_iter().map(GpuVertex::from).collect();
            let gpu_triangles: Vec<GpuTriangle> = geometry
                .triangles
                .into_iter()
                .map(GpuTriangle::from)
                .collect();

            if gpu_materials.len() != self.materials_storage.len() {
                self.materials_storage =
                    StorageBuffer::new_with_data(device, "materials_storage", &gpu_materials);
                update_bind_groups = true;
            } else {
                self.materials_storage.write(queue, &gpu_materials);
            }

            if gpu_vertices.len() != self.vertices_storage.len() {
                self.vertices_storage =
                    StorageBuffer::new_with_data(device, "vertices_storage", &gpu_vertices);
                update_bind_groups = true;
            } else {
                self.vertices_storage.write(queue, &gpu_vertices);
            }

            if gpu_triangles.len() != self.triangles_storage.len() {
                self.triangles_storage =
                    StorageBuffer::new_with_data(device, "triangles_storage", &gpu_triangles);
                update_bind_groups = true;
            } else {
                self.triangles_storage.write(queue, &gpu_triangles);
            }
        }

        if update_bind_groups {
            self.raytracing_bind_group = self.raytracing_pass.create_bind_group(
                device,
                &self.acc_input_texture,
                &self.acc_output_texture,
                &self.output_texture,
                &self.settings_uniform,
                &self.per_render_uniform,
                &self.camera_uniform,
                &self.materials_storage,
                &self.vertices_storage,
                &self.triangles_storage,
            );

            self.blit_bind_group = self
                .blit_pass
                .create_bind_group(device, &self.output_texture);
        }

        if self.num_samples < self.max_samples {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();

            self.per_render_uniform.write(
                queue,
                &[PerRenderUniform {
                    num_samples: self.num_samples,
                    current_time,
                    pad0: [0; 2],
                }],
            );

            self.raytracing_pass
                .dispatch(encoder, self.size, &self.raytracing_bind_group);

            encoder.copy_texture_to_texture(
                self.acc_output_texture.inner().as_image_copy(),
                self.acc_input_texture.inner().as_image_copy(),
                self.acc_output_texture.inner().size(),
            );

            self.num_samples += self.samples_per_render;
        }

        self.blit_pass.draw(encoder, view, &self.blit_bind_group);
    }
}

#[derive(Clone, Debug)]
pub struct RendererSettings {
    pub samples_per_render: u32,
    pub max_samples: u32,
    pub max_ray_depth: u32,
    pub furnace_test: bool,
}

impl Default for RendererSettings {
    fn default() -> Self {
        Self {
            samples_per_render: 1,
            max_samples: 1000,
            max_ray_depth: 10,
            furnace_test: false,
        }
    }
}

impl RendererSettings {
    pub fn validate(&self) -> bool {
        self.samples_per_render > 0 && self.max_ray_depth > 0 && self.max_samples > 0
    }
}

#[derive(Default)]
struct PreRenderCommands {
    reset: bool,
    resize: Option<glam::UVec2>,
    update_settings: Option<RendererSettings>,
    update_camera: Option<Camera>,
    update_geometry: Option<Geometry>,
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SettingsUniform {
    samples_per_render: u32,
    max_ray_depth: u32,
    furnace_test: u32,
    pad0: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PerRenderUniform {
    num_samples: u32,
    current_time: u32,
    pad0: [u32; 2],
}
