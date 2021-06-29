use std::{num::NonZeroU32, path::Path, sync::Arc};

use crate::render::BufferData;
use crate::{glm, util};
use bytemuck::{offset_of, Pod, Zeroable};

use super::{compile_shader_file, Buffer, DeviceContext, GBuffer, RenderContext, Size, Texture};

pub struct SSRTUniform {
    pub image_width: u32,
    pub image_height: u32,
    pub lod_width: u32,
    pub lod_height: u32,
    pub max_level: u32,
    pub near: f32,
    pub view_dir: glm::Vec3,
    pub eye_pos: glm::Vec3,
}
pub struct SSRTBindGroup {
    pub layout: wgpu::BindGroupLayout,
    pub buffer: Buffer<SSRTUniformData>,
    pub bindgroup: wgpu::BindGroup,
}
impl SSRTBindGroup {
    pub fn new(ctx: &DeviceContext) -> Self {
        let layout = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("ssrt.bindgroup"),
            });
        let buffer = Buffer::new_uniform_buffer(ctx, &[Default::default()], Some("ssrt.uniform"));
        let bindgroup = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            label: None,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.buffer.as_entire_binding(),
            }],
        });
        Self {
            layout,
            buffer,
            bindgroup,
        }
    }
}
#[derive(Clone, Copy, Default)]
pub struct SSRTUniformData {
    image_width: u32,
    image_height: u32,
    lod_width: u32,
    lod_height: u32,
    max_level: u32,
    near: f32,
    _pad0: [u32; 2],
    view_dir: [f32; 3],
    _pad1: f32,
    eye_pos: [f32; 3],
    _pad2: f32,
}
unsafe impl Zeroable for SSRTUniformData {}
unsafe impl Pod for SSRTUniformData {}
impl BufferData for SSRTUniformData {
    type Native = SSRTUniformData;
    fn new(value: &Self::Native) -> Self {
        assert!(offset_of!(SSRTUniformData, view_dir) % 4 == 0);
        assert!(offset_of!(SSRTUniformData, eye_pos) % 4 == 0);
        Self {
            image_width: value.image_width,
            image_height: value.image_height,
            lod_width: value.lod_width,
            lod_height: value.lod_height,
            max_level: value.max_level,
            near: value.near,
            view_dir: value.view_dir.into(),
            eye_pos: value.eye_pos.into(),
            ..Self::default()
        }
    }
}

pub struct DepthQuadTree {
    pub level: u32,
    pub textures: Vec<Texture>,
    pub pipeline: [wgpu::ComputePipeline; 2],
    pub bind_group_layout: [wgpu::BindGroupLayout; 2], // for building
    pub zquad_bind_group_layout: wgpu::BindGroupLayout, // for querying
    pub zquad_bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}
impl DepthQuadTree {
    pub fn new(ctx: &RenderContext, level: u32) -> Self {
        let size = ctx.size;
        let width = util::round_next_pow2(size.width) / 2;
        let height = util::round_next_pow2(size.height) / 2;
        let device = &ctx.device_ctx.device;
        let textures: Vec<Texture> = (0..level)
            .map(|lev| {
                let width = width >> lev;
                let height = height >> lev;
                Texture::create_color_attachment(
                    device,
                    &Size(width, height),
                    wgpu::TextureFormat::R32Float,
                    "depth quad",
                )
            })
            .collect();
        let cs = compile_shader_file(
            Path::new("src/shaders/ssgi.zquad.comp"),
            shaderc::ShaderKind::Compute,
            &device,
        )
        .unwrap();
        let zquad_layout0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: None,
        });
        let zquad_layout1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: None,
        });
        let pipeline0 = {
            let pipeline_layout =
                ctx.device_ctx
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("ssgi.pipeline.layout"),
                        bind_group_layouts: &[&zquad_layout0],
                        push_constant_ranges: &[wgpu::PushConstantRange {
                            stages: wgpu::ShaderStage::COMPUTE,
                            range: 0..20, // (image_size, width, height, level)
                        }],
                    });

            ctx.device_ctx
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ssgi.zquad.pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &cs,
                    entry_point: "main",
                })
        };
        let pipeline1 = {
            let pipeline_layout =
                ctx.device_ctx
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("ssgi.pipeline.layout"),
                        bind_group_layouts: &[&zquad_layout1],
                        push_constant_ranges: &[wgpu::PushConstantRange {
                            stages: wgpu::ShaderStage::COMPUTE,
                            range: 0..20,
                        }],
                    });

            ctx.device_ctx
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ssgi.zquad.pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &cs,
                    entry_point: "main",
                })
        };
        let zquad_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ssgi.trace.zquad.bindgroup.layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    // ty: wgpu::BindingType::StorageTexture {
                    //     access: wgpu::StorageTextureAccess::ReadOnly,
                    //     format: wgpu::TextureFormat::R32Float,
                    //     view_dimension: wgpu::TextureViewDimension::D2Array,
                    // },
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(level),
                }],
            });
        let zquad_views: Vec<_> = (0..level)
            .map(|lev| -> &wgpu::TextureView { &textures[lev as usize].view })
            .collect();
        let zquad_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &&zquad_bind_group_layout,
            label: Some("ssgi.trace.zquad.bindgroup"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&zquad_views[..]),
            }],
        });
        Self {
            level,
            textures,
            pipeline: [pipeline0, pipeline1],
            bind_group_layout: [zquad_layout0, zquad_layout1],
            zquad_bind_group_layout,
            zquad_bind_group,
            width,
            height,
        }
    }
}

impl DepthQuadTree {
    pub fn record_command(
        &mut self,
        ctx: &DeviceContext,
        gbuffer: &GBuffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for level in 0..self.level {
            let bind_group_layout = if level == 0 {
                &self.bind_group_layout[0]
            } else {
                &self.bind_group_layout[1]
            };
            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                label: None,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: if level == 0 {
                            wgpu::BindingResource::TextureView(&gbuffer.depth.view)
                        } else {
                            wgpu::BindingResource::TextureView(
                                &self.textures[level as usize - 1].view,
                            )
                        },
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.textures[level as usize].view,
                        ),
                    },
                ],
            });
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("zquad.pass"),
            });
            let pipeline = if level == 0 {
                &self.pipeline[0]
            } else {
                &self.pipeline[1]
            };
            let size = Size(gbuffer.depth.extent.width, gbuffer.depth.extent.height);
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.set_push_constants(
                0,
                bytemuck::cast_slice(&[size.0, size.1, self.width, self.height, level]),
            );
            compute_pass.dispatch(self.width / 16, self.height / 16, 1);
        }
    }
}
