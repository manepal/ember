/// A GPU texture with its view and sampler, ready for binding.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /// Create a texture from raw RGBA8 pixel data.
    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            width,
            height,
        }
    }

    /// Load a texture from an image file (PNG/JPEG).
    pub fn from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &std::path::Path,
        label: Option<&str>,
    ) -> Result<Self, String> {
        let img =
            image::open(path).map_err(|e| format!("Failed to open image {:?}: {}", path, e))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        Ok(Self::from_rgba(device, queue, &rgba, width, height, label))
    }

    /// Load a texture from in-memory image bytes (PNG/JPEG).
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
    ) -> Result<Self, String> {
        let img =
            image::load_from_memory(bytes).map_err(|e| format!("Failed to decode image: {}", e))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        Ok(Self::from_rgba(device, queue, &rgba, width, height, label))
    }

    /// Create the standard bind group layout for texture sampling.
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    /// Create a bind group from this texture for shader binding.
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}

/// Manages loaded textures and their GPU bind groups.
pub struct TextureStore {
    pub textures: Vec<Texture>,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl TextureStore {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = Texture::bind_group_layout(device);
        Self {
            textures: Vec::new(),
            bind_groups: Vec::new(),
            bind_group_layout,
        }
    }

    /// Add a texture and return its index.
    pub fn add(&mut self, device: &wgpu::Device, texture: Texture) -> usize {
        let bind_group = texture.create_bind_group(device, &self.bind_group_layout);
        let index = self.textures.len();
        self.textures.push(texture);
        self.bind_groups.push(bind_group);
        index
    }
}
