use std::{path::Path, sync::Arc};

use wgpu::util::DeviceExt;

use crate::render::{self, Buffer, BufferData, Camera, FrameContext, GPUScene, RenderContext, RenderPass, Size, Texture, UniformViewProjection, Vertex, ViewProjection, compile_shader_file};

pub struct ShadowPassInput {
    scene: Arc<GPUScene>,
    light_idx: u32,
    vp: ViewProjection,
    depth: Arc<Texture>,
}
pub struct ShadowPass {
    pipeline: wgpu::RenderPipeline,
    light_vp: Buffer<UniformViewProjection>,
    bindgroup: wgpu::BindGroup,
}

impl ShadowPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let device = &ctx.device_ctx.device;
        let mut compiler = shaderc::Compiler::new().unwrap();
        let fs = compile_shader_file(
            Path::new("src/shaders/shadow.frag"),
            shaderc::ShaderKind::Fragment,
            &ctx.device_ctx.device,
            &mut compiler,
        )
        .unwrap();
        let vs = compile_shader_file(
            Path::new("src/shaders/shadow.vert"),
            shaderc::ShaderKind::Vertex,
            &ctx.device_ctx.device,
            &mut compiler,
        )
        .unwrap();
        let light_vp = Buffer::new_uniform_buffer(
            &ctx.device_ctx,
            &[UniformViewProjection::default()],
            Some("light_view.vp"),
        );

        let bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[light_vp.bindgroup_layout_entry(
                0,
                wgpu::ShaderStage::VERTEX,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )],
        });
        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bindgroup_layout,
            entries: &[light_vp.bindgroup_entry(0)],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ShadowPass Pipeline Layout"),
            bind_group_layouts: &[&bindgroup_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ShadowPass pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: "main",
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
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
        Self {
            light_vp,
            pipeline,
            bindgroup,
        }
    }
}

impl RenderPass for ShadowPass {
    type Input = ShadowPassInput;
    fn record_command(
        &mut self,
        _size: Size,
        ctx: &mut RenderContext,
        _frame_ctx: &mut FrameContext,
        _camera: &dyn Camera,
        input: &Self::Input,
    ) -> wgpu::CommandBuffer {
        self.light_vp
            .upload(&ctx.device_ctx, &[UniformViewProjection::new(&input.vp)]);
        let mut encoder =
            ctx.device_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &input.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bindgroup, &[]);
            for m in &input.scene.meshes {
                render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                render_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..m.num_indices, 0, 0..1);
            }
        }
        encoder.finish()
    }
}
