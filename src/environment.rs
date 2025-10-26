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
        use image::codecs::hdr::HdrDecoder;
        use std::{fs, io::Cursor};

        let img_data = fs::read(path)?;

        let hdr_decoder = HdrDecoder::new(Cursor::new(img_data))?;
        let meta = hdr_decoder.metadata();
        let mut pixels = vec![[0.0, 0.0, 0.0, 0.0]; meta.width as usize * meta.height as usize];
        hdr_decoder.read_image_transform(
            |pix| {
                let rgb = pix.to_hdr();
                [rgb.0[0], rgb.0[1], rgb.0[2], 1.0f32]
            },
            &mut pixels[..],
        )?;

        Ok(Self {
            size: glam::uvec2(meta.width, meta.height),
            data: bytemuck::cast_slice(&pixels).to_vec(),
        })
    }

    pub fn validate(&self) -> bool {
        (self.size.x * self.size.y * std::mem::size_of::<glam::Vec4>() as u32) as usize
            == self.data.len()
    }
}
