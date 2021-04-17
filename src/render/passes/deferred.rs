use std::path::Path;

use wgpu::util::DeviceExt;

use crate::render::{
    compile_shader_file, Camera, FrameContext, RenderContext, RenderPass, Size, Vertex,
};

use super::{GBuffer, GBufferPassInput};

pub struct DeferredShadingPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
}

impl DeferredShadingPass {
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
            label: Some("DeferredShadingPass bind_group"),
        });
        let pipeline_laout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("DeferredShadingPass Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("DeferredShadingPass pipeline"),
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
impl RenderPass for DeferredShadingPass {
    type Input = GBufferPassInput;
    fn record_command(
        &mut self,
        _size: Size,
        ctx: &mut RenderContext,
        frame_ctx: &mut FrameContext,
        _camera: &dyn Camera,
        input: &Self::Input,
    ) -> wgpu::CommandBuffer {
        let mut encoder =
            ctx.device_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let bind_group = ctx
                .device_ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &input.gbuffer.normal.view,
                            ),
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
        encoder.finish()
    }
}
