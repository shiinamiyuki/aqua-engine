use std::sync::Arc;

use crate::render::{
    Buffer, BufferData, Camera, ColorAttachment, DeviceContext, FrameContext, GPUMesh, GPUScene,
    RenderContext, RenderPass, Size, Texture, UniformViewProjection, Vertex,
};

#[derive(Clone)]
pub struct GBuffer {
    pub depth: Arc<Texture>,
    pub normal: Arc<Texture>,
    pub world_pos: Arc<Texture>,
    pub layout: Arc<wgpu::BindGroupLayout>,
}
impl GBuffer {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = Texture::DEPTH_FORMAT;
    pub const NORMAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
    pub const WOLRD_POS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub fn bind_group_layout(ctx: &DeviceContext) -> wgpu::BindGroupLayout {
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("gbuffer.bindgroup.layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false
                        },
                        binding: 0,
                        count: None,
                        visibility: wgpu::ShaderStage::all(),
                    },
                    wgpu::BindGroupLayoutEntry {
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: Self::NORMAL_FORMAT,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        binding: 1,
                        count: None,
                        visibility: wgpu::ShaderStage::all(),
                    },
                    wgpu::BindGroupLayoutEntry {
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: Self::WOLRD_POS_FORMAT,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        binding: 2,
                        count: None,
                        visibility: wgpu::ShaderStage::all(),
                    },
                ],
            })
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
                    resource: wgpu::BindingResource::TextureView(&self.normal.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.world_pos.view),
                },
            ],
        })
    }
}

pub struct GBufferPass {
    pipeline: wgpu::RenderPipeline,
    camera_uniform: Buffer<UniformViewProjection>,
    bind_group0: wgpu::BindGroup,
}

pub struct GBufferPassInput {
    pub scene: Arc<GPUScene>,
    pub gbuffer: GBuffer,
}
impl GBufferPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let device = &ctx.device_ctx.device;
        let vs_src = std::fs::read_to_string("src/shaders/gbuffer.vert").unwrap();
        let fs_src = std::fs::read_to_string("src/shaders/gbuffer.frag").unwrap();
        let mut compiler = shaderc::Compiler::new().unwrap();
        let size = Size(ctx.size.width, ctx.size.height);
        let vs_spirv = compiler
            .compile_into_spirv(
                &vs_src,
                shaderc::ShaderKind::Vertex,
                "gbuffer.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                &fs_src,
                shaderc::ShaderKind::Fragment,
                "gbuffer.frag",
                "main",
                None,
            )
            .unwrap();
        let vs_data = wgpu::util::make_spirv(vs_spirv.as_binary_u8());
        let fs_data = wgpu::util::make_spirv(fs_spirv.as_binary_u8());
        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("GBufferPass Vertex Shader"),
            source: vs_data,
            flags: wgpu::ShaderFlags::default(),
        });
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("GBufferPass Fragment Shader"),
            source: fs_data,
            flags: wgpu::ShaderFlags::default(),
        });
        let camera_uniform = Buffer::<UniformViewProjection>::new_uniform_buffer(
            &ctx.device_ctx,
            &[UniformViewProjection::default()],
            None,
        );
        let bind_group_layout0 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[camera_uniform.bindgroup_layout_entry(
                    0,
                    wgpu::ShaderStage::VERTEX,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                )],
                label: Some("gbuffer.bindgroup_layout.0"),
            });
        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[camera_uniform.bindgroup_entry(0)],
            label: Some("gbuffer.bindgroup.0"),
            layout: &bind_group_layout0,
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout0],
                push_constant_ranges: &[],
            });
        let sc_desc = &ctx.sc_desc;
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GBuferPass Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",        // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &fs_module,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        // 4.
                        format: wgpu::TextureFormat::Rgba32Float,
                        alpha_blend: wgpu::BlendState::REPLACE,
                        color_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                    wgpu::ColorTargetState {
                        // 4.
                        format: wgpu::TextureFormat::Rgba32Float,
                        alpha_blend: wgpu::BlendState::REPLACE,
                        color_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });
        Self {
            pipeline: render_pipeline,
            camera_uniform,
            bind_group0,
        }
    }
}
impl RenderPass for GBufferPass {
    type Input = GBufferPassInput;
    fn record_command(
        &mut self,
        ctx: &mut RenderContext,
        frame_ctx: &mut FrameContext,
        camera: &dyn Camera,
        input: &Self::Input,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.camera_uniform.upload(
            &ctx.device_ctx,
            &[UniformViewProjection::new(
                &camera.build_view_projection_matrix(),
            )],
        );

        let color_attachments = vec![
            ColorAttachment {
                view: &input.gbuffer.normal.view,
            },
            ColorAttachment {
                view: &input.gbuffer.world_pos.view,
            },
        ];
        let attachment_descs: Vec<wgpu::RenderPassColorAttachmentDescriptor> = color_attachments
            .iter()
            .map(|color| color.get_descriptor())
            .collect();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &attachment_descs[..],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &input.gbuffer.depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, &self.bind_group0, &[]);

        for m in &input.scene.meshes {
            render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
            render_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..m.num_indices, 0, 0..1); // 2.
        }
    }
}
