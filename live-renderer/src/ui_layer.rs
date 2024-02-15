use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct UiLayer {
    context: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    visible: bool,
}

impl UiLayer {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        samples: u32,
    ) -> Self {
        let context = egui::Context::default();
        let state =
            egui_winit::State::new(context.clone(), context.viewport_id(), window, None, None);
        let renderer = egui_wgpu::Renderer::new(device, format, None, samples);
        Self {
            context,
            state,
            renderer,
            visible: true,
        }
    }

    pub fn using_mouse_or_keyboard(&self) -> bool {
        self.context.is_using_pointer()
            || self.context.wants_pointer_input()
            || self.context.wants_keyboard_input()
    }

    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) {
        if self.visible && self.state.on_window_event(window, event).consumed {
            return;
        }

        if let WindowEvent::KeyboardInput { event, .. } = event {
            if event.state == ElementState::Pressed
                && event.physical_key == PhysicalKey::Code(KeyCode::KeyU)
            {
                self.visible = !self.visible;
            }
        }
    }

    pub fn render(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        run_ui: impl FnOnce(&egui::Context),
    ) {
        if !self.visible {
            return;
        }

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: window.inner_size().into(),
            pixels_per_point: window.scale_factor() as f32,
        };

        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, run_ui);

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, window.scale_factor() as f32);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
