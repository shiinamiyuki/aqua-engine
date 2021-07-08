use std::{path::Path, sync::Arc};

use wgpu::util::DeviceExt;

use crate::render::{compile_shader_file, RenderContext, Texture};

use super::RenderPass;

pub struct PostProcessPass {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub vertex_buffer: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
    pub ctx: Arc<RenderContext>,
}

pub struct PostProcessPassDescriptor {
    pub ctx: Arc<RenderContext>,
}
pub struct PostProcessPassNode {}

pub struct PostProcessPassParams {
    pub color_buf: Arc<Texture>,
}
impl RenderPass for PostProcessPass {
    type Descriptor = PostProcessPassDescriptor;
    fn create_pass(desc: &Self::Descriptor) -> Self {
        let ctx = &desc.ctx;
        let device = &ctx.device_ctx.device;
        let fs = compile_shader_file(
            Path::new("src/shaders/deferred.frag"),
            "deferred.frag",
            shaderc::ShaderKind::Fragment,
            &ctx.device_ctx.device,
            None,
        )
        .unwrap();
        let vs = compile_shader_file(
            Path::new("src/shaders/deferred.vert"),
            "deferred.vert",
            shaderc::ShaderKind::Vertex,
            &ctx.device_ctx.device,
            None,
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
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                clamp_depth: false,
                conservative: false,
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
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
            ctx: ctx.clone(),
        }
    }
    type Params = PostProcessPassParams;
    type Node = PostProcessPassNode;
    fn record_command(
        &mut self,
        params: &Self::Params,
        frame_ctx: &crate::render::FrameContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self::Node {
        let bind_group = self
            .ctx
            .device_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&params.color_buf.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame_ctx.frame.view,
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
        Self::Node {}
    }
}
