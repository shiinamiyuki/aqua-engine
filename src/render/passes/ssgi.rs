use std::{path::Path, sync::Arc};

use nalgebra_glm::Vec3;
use rand::Rng;
use wgpu::util::DeviceExt;

use crate::render::{
    compile_shader_file, Buffer, BufferData, Camera, ComputePass, CubeMap, DeviceContext,
    FrameContext, GPUScene, PointLight, PointLightData, RenderContext, RenderPass, Scene, Size,
    Texture, UniformViewProjection, Vertex, ViewProjection,
};

use super::{GBuffer, GBufferPass, GBufferPassInput};

struct DepthQuadTree {
    level: u32,
    textures: Vec<Texture>,
    pipeline: [wgpu::ComputePipeline; 2],
    bind_group_layout: [wgpu::BindGroupLayout; 2],
    width: u32,
}
impl DepthQuadTree {
    fn new(ctx: &RenderContext, level: u32) -> Self {
        let size = ctx.size;
        let size = size.width.max(size.height);
        let size = {
            let mut i = 2;
            while i < size {
                i *= 2;
            }
            i / 2
        };
        let device = &ctx.device_ctx.device;
        let textures = (0..level)
            .map(|lev| {
                let size = size / (2u32.pow(lev));
                Texture::create_color_attachment(
                    device,
                    &Size(size, size),
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
                            range: 0..16, // (image_size, width, level)
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
                            range: 0..16, // (image_size, width, level)
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
        Self {
            level,
            textures,
            pipeline: [pipeline0, pipeline1],
            bind_group_layout: [zquad_layout0, zquad_layout1],
            width: size,
        }
    }
}

impl ComputePass for DepthQuadTree {
    type Input = GBuffer;
    fn record_command(
        &mut self,
        size: Size,
        ctx: &mut DeviceContext,
        input: &Self::Input,
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
                            wgpu::BindingResource::TextureView(&input.depth.view)
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
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.set_push_constants(
                0,
                bytemuck::cast_slice(&[size.0, size.1, self.width, level]),
            )
        }
    }
}
pub struct SSGIPass {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    light: Buffer<PointLightData>,
    seeds: wgpu::Buffer,
    camera_uniform: Buffer<UniformViewProjection>,
    camera_bindgroup: wgpu::BindGroup,
    depth_quad_tree: DepthQuadTree,
    // light_vp: Buffer<UniformViewProjection>,
}

impl SSGIPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let device = &ctx.device_ctx.device;
        let cs = compile_shader_file(
            Path::new("src/shaders/ssgi.trace.comp"),
            shaderc::ShaderKind::Compute,
            &ctx.device_ctx.device,
        )
        .unwrap();
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("ssgi.bindgroup.layout"),
        });
        let camera_uniform = Buffer::<UniformViewProjection>::new_uniform_buffer(
            &ctx.device_ctx,
            &[UniformViewProjection::default()],
            None,
        );
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[camera_uniform.bindgroup_layout_entry(
                    0,
                    wgpu::ShaderStage::COMPUTE,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                )],
                label: Some("ssgi.bindgroup_layout.2"),
            });
        let camera_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[camera_uniform.bindgroup_entry(0)],
            label: Some("ssgi.bindgroup.2"),
            layout: &camera_bind_group_layout,
        });
        let pipeline_layout =
            ctx.device_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ssgi.pipeline.layout"),
                    bind_group_layouts: &[
                        &bind_group_layout,
                        &GBuffer::bind_group_layout(&ctx.device_ctx),
                        &camera_bind_group_layout,
                    ],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStage::COMPUTE,
                        range: 0..(4 * 4 * 3),
                    }],
                });
        let pipeline =
            ctx.device_ctx
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ssgi.pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &cs,
                    entry_point: "main",
                });
        // let light_vp = Buffer::new_uniform_buffer(
        //     &ctx.device_ctx,
        //     &[UniformViewProjection::default(); 6],
        //     Some("light_view.vp"),
        // );
        let mut rng = rand::thread_rng();
        let seeds_data: Vec<u32> = (0..(1920 * 1080)).map(|_| rng.gen::<u32>()).collect();
        let seeds = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("seeds"),
            contents: bytemuck::cast_slice(&seeds_data[..]),
            usage: wgpu::BufferUsage::STORAGE
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::UNIFORM,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            light: Buffer::new_uniform_buffer(
                &ctx.device_ctx,
                &[PointLightData::default()],
                Some("ssgi.point_light"),
            ), // light_vp,
            seeds,
            camera_bindgroup,
            camera_uniform,
            depth_quad_tree: DepthQuadTree::new(ctx, 4),
        }
    }
}
pub struct SSGIPassInput {
    pub scene: Arc<GPUScene>,
    pub light_idx: u32,
    pub cubemap: Arc<CubeMap>,
    pub gbuffer: GBuffer,
    pub color: Arc<Texture>,
    pub view_dir: Vec3,
    pub eye_pos: Vec3,
    pub vp: ViewProjection,
}
impl RenderPass for SSGIPass {
    type Input = SSGIPassInput;
    fn record_command(
        &mut self,
        ctx: &mut RenderContext,
        _frame_ctx: &mut FrameContext,
        _camera: &dyn Camera,
        input: &Self::Input,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.depth_quad_tree.record_command(
            Size(ctx.size.width, ctx.size.height),
            &mut ctx.device_ctx,
            &input.gbuffer,
            encoder,
        );
        self.light.upload(
            &ctx.device_ctx,
            &[PointLightData::new(
                &input.scene.point_lights[input.light_idx as usize],
            )],
        );
        let bindgroup0 = ctx
            .device_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ssgi.bindgroup0"),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input.color.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&input.cubemap.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.light.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.seeds.as_entire_binding(),
                    },
                ],
                layout: &self.bind_group_layout,
            });
        let bindgroup1 = input.gbuffer.create_bind_group(&ctx.device_ctx);
        self.camera_uniform
            .upload(&ctx.device_ctx, &[UniformViewProjection::new(&input.vp)]);
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("SSGI pass"),
        });
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_push_constants(
            0,
            bytemuck::cast_slice(&[
                input.light_idx as i32,
                input.color.extent.width as i32,
                input.color.extent.height as i32,
            ]),
        );
        // println!("{}", input.view_dir);
        compute_pass.set_push_constants(4 * 4, bytemuck::cast_slice(input.view_dir.as_slice()));
        compute_pass.set_push_constants(
            4 * 4 + 4 * 4,
            bytemuck::cast_slice(input.eye_pos.as_slice()),
        );
        compute_pass.set_bind_group(0, &bindgroup0, &[]);
        compute_pass.set_bind_group(1, &bindgroup1, &[]);
        compute_pass.set_bind_group(2, &self.camera_bindgroup, &[]);
        compute_pass.dispatch(
            (input.color.extent.width + 15) / 16,
            (input.color.extent.height + 15) / 16,
            1,
        );
    }
}
