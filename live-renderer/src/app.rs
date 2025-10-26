use std::time::Instant;

use light_raytracer::{Camera, Environment, Geometry, Renderer, RendererSettings};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{camera_controller::CameraController, ui_layer::UiLayer, wgpu_context::WgpuContext};

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Live Renderer")
        .with_inner_size(PhysicalSize {
            width: 1800,
            height: 1200,
        })
        .build(&event_loop)
        .unwrap();
    let mut app = App::new(window).await;

    println!("Hello world");

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut last_tick = Instant::now();
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == app.window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();
                    let dt = (now - last_tick).as_millis() as f32 / 1000.0;
                    last_tick = now;

                    app.update(dt);
                    app.render();
                }
                _ => app.on_window_event(event),
            },
            Event::DeviceEvent { ref event, .. } => app.on_device_event(event),
            Event::AboutToWait => app.window.request_redraw(),
            _ => {}
        })
        .unwrap();
}

struct App {
    window: Window,
    wgpu_context: WgpuContext,
    camera: Camera,
    camera_controller: CameraController,
    renderer_settings: RendererSettings,
    renderer: Renderer,
    ui_layer: UiLayer,
    frame_time: f32,
}

impl App {
    async fn new(window: Window) -> Self {
        let wgpu_context = WgpuContext::new(&window).await;

        let size = glam::uvec2(window.inner_size().width, window.inner_size().height);

        let device = wgpu_context.device();
        let queue = wgpu_context.queue();
        let format = wgpu_context.surface_config().format;

        let renderer_settings = RendererSettings::default();

        let camera = Camera {
            position: glam::vec3(0.0, 0.0, 8.0),
            forward: glam::vec3(0.0, 0.0, -1.0),
            aspect: size.x as f32 / size.y as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        };

        let camera_controller = CameraController::new(4.0, 0.1);

        let environment = Environment::load("assets/rural_crossroads_1k.hdr").unwrap();

        let geometry = Geometry::load("assets/basic.gltf", "Scene").unwrap();

        let renderer = Renderer::new(
            device,
            queue,
            size,
            format,
            renderer_settings.clone(),
            camera.clone(),
            environment,
            geometry,
        );

        let ui_layer = UiLayer::new(&window, device, format, 1);

        Self {
            window,
            wgpu_context,
            camera,
            camera_controller,
            renderer_settings,
            renderer,
            ui_layer,
            frame_time: 0.0,
        }
    }

    fn on_window_event(&mut self, event: &WindowEvent) {
        self.ui_layer.on_window_event(&self.window, event);

        match *event {
            WindowEvent::MouseInput { state, button, .. }
                if !self.ui_layer.using_mouse_or_keyboard() =>
            {
                self.camera_controller.on_mouse_event(button, state);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } if !self.ui_layer.using_mouse_or_keyboard() => {
                self.camera_controller.on_key_event(physical_key, state);
            }
            WindowEvent::Resized(new_size) => {
                let new_size = glam::uvec2(new_size.width, new_size.height);

                self.wgpu_context.resize(new_size);
                self.renderer.resize(new_size);
                self.camera.aspect = new_size.x as f32 / new_size.y as f32;
                self.renderer.update_camera(self.camera.clone());
            }
            _ => {}
        }
    }

    fn on_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if !self.ui_layer.using_mouse_or_keyboard() {
                self.camera_controller
                    .on_mouse_motion(glam::vec2(delta.0 as f32, delta.1 as f32));
            }
        }
    }

    fn update(&mut self, dt: f32) {
        self.frame_time = dt;
        if self
            .camera_controller
            .update(dt, &self.window, &mut self.camera)
        {
            self.renderer.update_camera(self.camera.clone());
        }
    }

    fn render(&mut self) {
        let device = self.wgpu_context.device();
        let queue = self.wgpu_context.queue();
        let surface = self.wgpu_context.surface();

        let frame = surface
            .get_current_texture()
            .expect("failed to acquire surface texture");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        self.renderer
            .render(device, queue, &mut encoder, &frame_view);

        self.ui_layer.render(
            &self.window,
            device,
            queue,
            &mut encoder,
            &frame_view,
            |ctx| {
                egui::Window::new("Main").title_bar(false).show(ctx, |ui| {
                    ui.heading("Statistics");
                    ui.label(format!("Frame Time: {:.2} s", self.frame_time));
                    ui.label(format!(
                        "Frames per Second: {}",
                        (1.0 / self.frame_time) as u32
                    ));
                    if ui.button("Reset").clicked() {
                        self.renderer.reset();
                    }

                    ui.separator();

                    ui.heading("Settings");

                    egui::Grid::new("Settings")
                        .num_columns(2)
                        .spacing([15.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Samples Per Render");
                            let mut changed = ui
                                .add(
                                    egui::DragValue::new(
                                        &mut self.renderer_settings.samples_per_render,
                                    )
                                    .clamp_range(1..=100),
                                )
                                .changed();
                            ui.end_row();

                            ui.label("Max Samples");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut self.renderer_settings.max_samples)
                                        .clamp_range(1..=1000000),
                                )
                                .changed();
                            ui.end_row();

                            ui.label("Max Ray Depth");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut self.renderer_settings.max_ray_depth)
                                        .clamp_range(1..=100),
                                )
                                .changed();
                            ui.end_row();

                            ui.label("Furnace Test");
                            changed |= ui
                                .add(egui::Checkbox::new(
                                    &mut self.renderer_settings.furnace_test,
                                    "",
                                ))
                                .changed();
                            ui.end_row();

                            ui.label("Environment Brightness");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(
                                        &mut self.renderer_settings.environment_brightness,
                                    )
                                    .speed(0.01)
                                    .fixed_decimals(2)
                                    .clamp_range(0.0..=10.0),
                                )
                                .changed();
                            ui.end_row();

                            if changed {
                                self.renderer
                                    .update_settings(self.renderer_settings.clone());
                            }
                        });
                });
            },
        );

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
