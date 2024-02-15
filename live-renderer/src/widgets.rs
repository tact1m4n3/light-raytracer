pub struct Vec2Drag<'a>(&'a mut glam::Vec2);

impl<'a> Vec2Drag<'a> {
    pub fn new(value: &'a mut glam::Vec2) -> Self {
        Self(value)
    }
}

impl<'a> egui::Widget for Vec2Drag<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut response = ui
            .horizontal(|ui| {
                if ui.label("X").clicked() {
                    self.0.x = 0.0;
                    changed = true;
                }

                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.0.x)
                            .max_decimals(2)
                            .speed(0.01),
                    )
                    .changed();

                if ui.label("Y").clicked() {
                    self.0.y = 0.0;
                    changed = true;
                }

                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.0.y)
                            .max_decimals(2)
                            .speed(0.01),
                    )
                    .changed();
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

pub struct Vec3Drag<'a>(&'a mut glam::Vec3);

impl<'a> Vec3Drag<'a> {
    pub fn new(value: &'a mut glam::Vec3) -> Self {
        Self(value)
    }
}

impl<'a> egui::Widget for Vec3Drag<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut response = ui
            .horizontal(|ui| {
                if ui.label("X").clicked() {
                    self.0.x = 0.0;
                    changed = true;
                }

                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.0.x)
                            .max_decimals(2)
                            .speed(0.01),
                    )
                    .changed();

                if ui.label("Y").clicked() {
                    self.0.y = 0.0;
                    changed = true;
                }

                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.0.y)
                            .max_decimals(2)
                            .speed(0.01),
                    )
                    .changed();

                if ui.label("Z").clicked() {
                    self.0.z = 0.0;
                    changed = true;
                }

                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.0.z)
                            .max_decimals(2)
                            .speed(0.01),
                    )
                    .changed();
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }
}
