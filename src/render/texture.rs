use std::num::NonZeroU32;

use super::Size;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub view: wgpu::TextureView,
    pub extent: wgpu::Extent3d,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.
    pub fn create_depth_texture_from_sc(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        label: &str,
    ) -> Self {
        Self::create_depth_texture_with_size(&device, &Size(sc_desc.width, sc_desc.height), label)
    }

    pub fn create_color_attachment(
        device: &wgpu::Device,
        size: &Size,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers:1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE,
        };
        let texture = device.create_texture(&desc);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None, // 5.
            ..Default::default()
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            sampler,
            view,
            extent: size,
        }
    }
    pub fn create_depth_texture_with_size(device: &wgpu::Device, size: &Size, label: &str) -> Self {
        let size = wgpu::Extent3d {
            // 2.
            width: size.0,  //sc_desc.width,
            height: size.1, //sc_desc.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsage::SAMPLED,
        };
        let texture = device.create_texture(&desc);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            sampler,
            view,
            extent: size,
        }
    }
}

pub struct CubeMap {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub face_views: Vec<wgpu::TextureView>,
    pub view: wgpu::TextureView,
    pub extent: wgpu::Extent3d,
}

impl CubeMap {
    pub fn create_cubemap(
        device: &wgpu::Device,
        res: u32,
        format: wgpu::TextureFormat,
        label: &str,
        depth_map: bool,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: res,
            height: res,
            depth_or_array_layers: 6,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        };
        let texture = device.create_texture(&desc);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: if depth_map {
                Some(wgpu::CompareFunction::LessEqual)
            } else {
                None
            },
            ..Default::default()
        });
        let face_views: Vec<wgpu::TextureView> = (0..6u32)
            .into_iter()
            .map(|i| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i,
                    array_layer_count: NonZeroU32::new(1),
                    ..Default::default()
                })
            })
            .collect();
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        Self {
            texture,
            sampler,
            face_views,
            view,
            extent: size,
        }
    }
}
