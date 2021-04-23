use std::{path::Path, sync::Arc};

use wgpu::util::DeviceExt;

use crate::render::{
    compile_shader_file, Buffer, BufferData, Camera, CubeMap, FrameContext, GPUScene, PointLight,
    PointLightData, RenderContext, RenderPass, Scene, Size, Texture, UniformViewProjection, Vertex,
};

use super::{GBuffer, GBufferPass, GBufferPassInput, ShadowPass, ShadowPassInput};

pub struct ShadowMapPass {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    light: Buffer<PointLightData>,
    // light_vp: Buffer<UniformViewProjection>,
}

impl ShadowMapPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let device = &ctx.device_ctx.device;
        let mut compiler = shaderc::Compiler::new().unwrap();
        let cs = compile_shader_file(
            Path::new("src/shaders/shadow_map.comp"),
            shaderc::ShaderKind::Compute,
            &ctx.device_ctx.device,
            &mut compiler,
        )
        .unwrap();
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            ],
            label: Some("shadow_map.bindgroup.layout"),
        });
        let pipeline_layout =
            ctx.device_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shadow_map.pipeline.layout"),
                    bind_group_layouts: &[
                        &bind_group_layout,
                        &GBuffer::bind_group_layout(&ctx.device_ctx),
                    ],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStage::COMPUTE,
                        range: 0..12,
                    }],
                });
        let pipeline =
            ctx.device_ctx
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("shadow_map.pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &cs,
                    entry_point: "main",
                });
        // let light_vp = Buffer::new_uniform_buffer(
        //     &ctx.device_ctx,
        //     &[UniformViewProjection::default(); 6],
        //     Some("light_view.vp"),
        // );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        Self {
            pipeline,
            bind_group_layout,
            sampler,
            light: Buffer::new_uniform_buffer(
                &ctx.device_ctx,
                &[PointLightData::default()],
                Some("shadow_map.point_light"),
            ), // light_vp,
        }
    }
}
pub struct ShadowMapPassInput {
    pub scene: Arc<GPUScene>,
    pub light_idx: u32,
    pub cubemap: Arc<CubeMap>,
    pub gbuffer: GBuffer,
    pub color: Arc<Texture>,
}
impl RenderPass for ShadowMapPass {
    type Input = ShadowMapPassInput;
    fn record_command(
        &mut self,
        ctx: &mut RenderContext,
        _frame_ctx: &mut FrameContext,
        _camera: &dyn Camera,
        input: &Self::Input,
        encoder: &mut wgpu::CommandEncoder,
    ) {
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
                label: Some("shadow_map.bindgroup0"),
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
                ],
                layout: &self.bind_group_layout,
            });
        let bindgroup1 = input.gbuffer.create_bind_group(&ctx.device_ctx);
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Shadow Map pass"),
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

        compute_pass.set_bind_group(0, &bindgroup0, &[]);
        compute_pass.set_bind_group(1, &bindgroup1, &[]);

        compute_pass.dispatch(
            (input.color.extent.width + 15) / 16,
            (input.color.extent.height + 15) / 16,
            1,
        );
    }
}
pub struct RenderFrameBufferPass {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub vertex_buffer: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
}

impl RenderFrameBufferPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let device = &ctx.device_ctx.device;
        let mut compiler = shaderc::Compiler::new().unwrap();
        let fs = compile_shader_file(
            Path::new("src/shaders/deferred.frag"),
            shaderc::ShaderKind::Fragment,
            &ctx.device_ctx.device,
            &mut compiler,
        )
        .unwrap();
        let vs = compile_shader_file(
            Path::new("src/shaders/deferred.vert"),
            shaderc::ShaderKind::Vertex,
            &ctx.device_ctx.device,
            &mut compiler,
        )
        .unwrap();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
            ],
            label: Some("RenderFrameBufferPass bind_group"),
        });
        let pipeline_laout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("RenderFrameBufferPass Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RenderFrameBufferPass pipeline"),
            layout: Some(&pipeline_laout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float3,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });
        /*
        -2, 1       1, 1


                    1, -2


        */
        let vertices: [[f32; 3]; 3] = [[-2.0, 1.0, 0.0], [1.0, -2.0, 0.0], [1.0, 1.0, 0.0]];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Full Screen Triangle"),
            contents: bytemuck::cast_slice(&vertices[..]),
            usage: wgpu::BufferUsage::VERTEX,
        });
        Self {
            pipeline,
            bind_group_layout,
            vertex_buffer,
            sampler,
        }
    }
}
impl RenderPass for RenderFrameBufferPass {
    type Input = Arc<Texture>;
    fn record_command(
        &mut self,
        ctx: &mut RenderContext,
        frame_ctx: &mut FrameContext,
        camera: &dyn Camera,
        input: &Self::Input,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let bind_group = ctx
            .device_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame_ctx.frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}
pub struct DeferredShadingPass {
    shadow_pass: ShadowPass,
    shadow_map_pass: ShadowMapPass,
    gbuffer_pass: GBufferPass,
    render_frame_buffer_pass: RenderFrameBufferPass,
    gbuffer: GBuffer,
    shadow_cube_map: Arc<CubeMap>,
    color_buffer: Arc<Texture>,
}

impl DeferredShadingPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let gbuffer = GBuffer {
            depth: Arc::new(Texture::create_depth_texture_from_sc(
                &ctx.device_ctx.device,
                &ctx.sc_desc,
                "gbuffer.depth",
            )),
            normal: Arc::new(Texture::create_color_attachment(
                &ctx.device_ctx.device,
                &Size(ctx.sc_desc.width, ctx.sc_desc.height),
                wgpu::TextureFormat::Rgba32Float,
                "gbuffer.normal",
            )),
            world_pos: Arc::new(Texture::create_color_attachment(
                &ctx.device_ctx.device,
                &Size(ctx.sc_desc.width, ctx.sc_desc.height),
                wgpu::TextureFormat::Rgba32Float,
                "gbuffer.world_pos",
            )),
            layout: Arc::new(GBuffer::bind_group_layout(&ctx.device_ctx)),
        };
        let shadow_cube_map = Arc::new(CubeMap::create_cubemap(
            &ctx.device_ctx.device,
            128,
            wgpu::TextureFormat::R32Float,
            "omni-shadow",
            true,
        ));
        let color_buffer = Arc::new(Texture::create_color_attachment(
            &ctx.device_ctx.device,
            &Size(ctx.sc_desc.width, ctx.sc_desc.height),
            wgpu::TextureFormat::Rgba32Float,
            "deferred.color",
        ));
        Self {
            gbuffer,
            shadow_cube_map,
            shadow_pass: ShadowPass::new(ctx, 128),
            shadow_map_pass: ShadowMapPass::new(ctx),
            gbuffer_pass: GBufferPass::new(ctx),
            render_frame_buffer_pass: RenderFrameBufferPass::new(ctx),
            color_buffer,
        }
    }
}
pub struct DeferredShadingInput {
    pub scene: Arc<GPUScene>,
}
impl RenderPass for DeferredShadingPass {
    type Input = DeferredShadingInput;
    fn record_command(
        &mut self,
        ctx: &mut RenderContext,
        frame_ctx: &mut FrameContext,
        camera: &dyn Camera,
        input: &Self::Input,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        {
            let input = ShadowPassInput {
                scene: input.scene.clone(),
                light_idx: 0,
                cubemap: self.shadow_cube_map.clone(),
            };
            self.shadow_pass
                .record_command(ctx, frame_ctx, camera, &input, encoder);
        }
        {
            let input = GBufferPassInput {
                scene: input.scene.clone(),
                gbuffer: self.gbuffer.clone(),
            };
            {
                self.gbuffer_pass
                    .record_command(ctx, frame_ctx, camera, &input, encoder);
            }
        }
        {
            let input = ShadowMapPassInput {
                scene: input.scene.clone(),
                light_idx: 0,
                cubemap: self.shadow_cube_map.clone(),
                gbuffer: self.gbuffer.clone(),
                color: self.color_buffer.clone(),
            };
            self.shadow_map_pass
                .record_command(ctx, frame_ctx, camera, &input, encoder)
        }
        {
            self.render_frame_buffer_pass.record_command(
                ctx,
                frame_ctx,
                camera,
                &self.color_buffer,
                encoder,
            );
        }
    }
}
