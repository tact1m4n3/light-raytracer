pub struct Texture2D {
    inner: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture2D {
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        size: glam::UVec2,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        mip_level_count: u32,
    ) -> Self {
        let inner = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = inner.create_view(&wgpu::TextureViewDescriptor {
            label: Some(label),
            format: None,
            dimension: Some(if mip_level_count == 1 {
                wgpu::TextureViewDimension::D2
            } else {
                wgpu::TextureViewDimension::D2Array
            }),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(mip_level_count),
            base_array_layer: 0,
            array_layer_count: None,
        });

        Self { inner, view }
    }

    pub fn inner(&self) -> &wgpu::Texture {
        &self.inner
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
