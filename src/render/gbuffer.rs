use std::sync::Arc;

use crate::render::{DeviceContext, Size, Texture};

/*
GBuffer Layout

rt0: rgb:color, a: metallic
rt1: rgb:normal, b: roughness
rt2: rgb:world pos



*/
#[derive(Clone, Copy, Debug)]
pub struct GBufferOptions {
    pub hdr: bool,
    pub aov: bool,
}
impl Default for GBufferOptions {
    fn default() -> Self {
        Self {
            hdr: true,
            aov: false,
        }
    }
}
// #[derive(Clone)]
pub struct GBuffer {
    pub render_targets: Vec<Arc<Texture>>,
    pub formats: Vec<wgpu::TextureFormat>,
    pub layout: wgpu::BindGroupLayout,
    pub depth: Arc<Texture>,
}
impl GBuffer {
    pub fn num_render_targets(&self)->u32 {
        self.render_targets.len() as u32
    }
    // pub const DEPTH_FORMAT: wgpu::TextureFormat = Texture::DEPTH_FORMAT;
    // pub const NORMAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
    // pub const WOLRD_POS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
    pub fn rt_formats(options: &GBufferOptions) -> Vec<wgpu::TextureFormat> {
        if options.hdr {
            if options.aov {
                vec![
                    wgpu::TextureFormat::Rgba32Float,
                    wgpu::TextureFormat::Rgba32Float,
                    wgpu::TextureFormat::Rgba32Float,
                    wgpu::TextureFormat::Rgba32Float,
                ]
            } else {
                vec![
                    wgpu::TextureFormat::Rgba32Float,
                    wgpu::TextureFormat::Rgba32Float,
                    wgpu::TextureFormat::Rgba32Float,
                ]
            }
        } else {
            unimplemented!()
        }
    }
    pub fn bind_group_layout(
        ctx: &DeviceContext,
        options: &GBufferOptions,
    ) -> wgpu::BindGroupLayout {
        let formats = Self::rt_formats(options);
        let mut entires: Vec<_> = formats
            .iter()
            .enumerate()
            .map(|(i, format)| wgpu::BindGroupLayoutEntry {
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    format: formats[i],
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                binding: 1 + i as u32,
                count: None,
                visibility: wgpu::ShaderStage::all(),
            })
            .collect();
        entires.insert(
            0,
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
        );
        let layout = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("gbuffer.bindgroup.layout"),
                entries: entires.as_slice(),
            });
        layout
    }
    pub fn new(ctx: &DeviceContext, size: &Size, options: &GBufferOptions) -> Self {
        let formats = Self::rt_formats(options);
        let layout = Self::bind_group_layout(ctx, options);
        let render_targets: Vec<_> = (0..formats.len())
            .map(|i| {
                let label = format!("gbuffer.rt{}", i);
                Arc::new(Texture::create_color_attachment(
                    &ctx.device,
                    size,
                    formats[i],
                    &label,
                ))
            })
            .collect();
        Self {
            formats,
            depth: Arc::new(Texture::create_depth_texture_with_size(
                &ctx.device,
                size,
                "gbuffer.depth",
            )),
            render_targets,
            layout,
        }
    }
    pub fn create_bind_group(&self, ctx: &DeviceContext) -> wgpu::BindGroup {
        let mut entries:Vec<_> = self.render_targets.iter().enumerate().map(|(i, rt)|{
            wgpu::BindGroupEntry {
                binding: i as u32 + 1,
                resource: wgpu::BindingResource::TextureView(&rt.view),
            }
        }).collect();
        entries.insert(0, wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.depth.view),
        });
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gbuffer.bindgroup"),
            layout: &self.layout,
            entries: entries.as_slice(),
        })
    }
}
