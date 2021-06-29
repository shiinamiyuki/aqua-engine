use std::sync::Arc;

use crate::render::{DeviceContext, Size, Texture};

/*
GBuffer Layout

rt0: rgb:color, a: metallic
rt1: rgb:normal, b: roughness
rt2: rgb:world pos



*/
// #[derive(Clone)]
pub struct GBuffer {
    pub render_targets: [Arc<Texture>; 3],
    pub formats: [wgpu::TextureFormat; 3],
    pub layout: wgpu::BindGroupLayout,
    pub depth: Arc<Texture>,
}
impl GBuffer {
    // pub const DEPTH_FORMAT: wgpu::TextureFormat = Texture::DEPTH_FORMAT;
    // pub const NORMAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
    // pub const WOLRD_POS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
    pub fn rt_formats(hdr: bool) -> [wgpu::TextureFormat; 3] {
        if hdr {
            [
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureFormat::Rgba32Float,
            ]
        } else {
            unimplemented!()
        }
    }
    pub fn bind_group_layout(ctx: &DeviceContext, hdr: bool) -> wgpu::BindGroupLayout {
        let formats = Self::rt_formats(true);
        let layout = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("gbuffer.bindgroup.layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        binding: 0,
                        count: None,
                        visibility: wgpu::ShaderStage::all(),
                    },
                    wgpu::BindGroupLayoutEntry {
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: formats[0],
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        binding: 1,
                        count: None,
                        visibility: wgpu::ShaderStage::all(),
                    },
                    wgpu::BindGroupLayoutEntry {
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: formats[1],
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        binding: 2,
                        count: None,
                        visibility: wgpu::ShaderStage::all(),
                    },
                    wgpu::BindGroupLayoutEntry {
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: formats[2],
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        binding: 3,
                        count: None,
                        visibility: wgpu::ShaderStage::all(),
                    },
                ],
            });
        layout
    }
    pub fn new(ctx: &DeviceContext, size: &Size) -> Self {
        let formats = Self::rt_formats(true);
        let layout = Self::bind_group_layout(ctx, true);
        Self {
            formats,
            depth: Arc::new(Texture::create_depth_texture_with_size(
                &ctx.device,
                size,
                "gbuffer.depth",
            )),
            render_targets: [
                Arc::new(Texture::create_color_attachment(
                    &ctx.device,
                    size,
                    formats[0],
                    "gbuffer.rt0",
                )),
                Arc::new(Texture::create_color_attachment(
                    &ctx.device,
                    size,
                    formats[1],
                    "gbuffer.rt1",
                )),
                Arc::new(Texture::create_color_attachment(
                    &ctx.device,
                    size,
                    formats[2],
                    "gbuffer.rt2",
                )),
            ],
            layout,
        }
    }
    pub fn create_bind_group(&self, ctx: &DeviceContext) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gbuffer.bindgroup"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.render_targets[0].view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.render_targets[1].view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.render_targets[2].view),
                },
            ],
        })
    }
}
