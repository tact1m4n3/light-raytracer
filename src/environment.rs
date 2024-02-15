pub struct Environment {
    pub size: glam::UVec2,
    pub data: Vec<u8>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            size: glam::uvec2(1, 1),
            data: bytemuck::cast_slice(&[glam::vec4(0.2, 0.3, 0.4, 1.0)]).to_owned(),
        }
    }
}

impl Environment {
    #[cfg(feature = "image")]
    pub fn load(path: &str) -> Result<Self, image::ImageError> {
        use image::EncodableLayout;

        let img = image::open(path)?;
        let size = glam::uvec2(img.width(), img.height());
        let data = img.into_rgba32f().as_bytes().to_owned();
        Ok(Self { size, data })
    }

    pub fn validate(&self) -> bool {
        (self.size.x * self.size.y * std::mem::size_of::<glam::Vec4>() as u32) as usize
            == self.data.len()
    }
}
