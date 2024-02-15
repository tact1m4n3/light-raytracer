mod app;
mod camera_controller;
mod ui_layer;
mod wgpu_context;
mod widgets;

fn main() {
    env_logger::init();
    pollster::block_on(app::run());
}
