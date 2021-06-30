use std::{
    cell::{Cell, RefCell},
    path::Path,
    sync::Arc,
    usize,
};

use shaderc::CompileOptions;

use crate::render::{
    compile_shader_file, Buffer, BufferData, Camera, DeviceContext, FrameContext, GBuffer,
    GBufferOptions, GPUMesh, GPUScene, RenderContext, Size, Texture, UniformViewProjection, Vertex,
};

use super::RenderPass;

pub struct GBufferPass {
    pipeline: wgpu::RenderPipeline,
    camera_uniform: Buffer<UniformViewProjection>,
    bind_group0: wgpu::BindGroup,
    ctx: Arc<RenderContext>,
}

pub struct GBufferPassParams {
    pub scene: Arc<GPUScene>,
    pub gbuffer: Arc<GBuffer>,
    pub camera: Camera,
}
pub struct GBufferPassDescriptor {
    pub ctx: Arc<RenderContext>,
    pub options: GBufferOptions,
}

pub struct GBufferPassNode {
    pub gbuffer: Arc<GBuffer>,
}
impl RenderPass for GBufferPass {
    type Descriptor = GBufferPassDescriptor;
    type Params = GBufferPassParams;
    type Node = GBufferPassNode;
    fn create_pass(desc: &Self::Descriptor) -> Self {
        let ctx = &desc.ctx;
        let device = &ctx.device_ctx.device;
        let (vs, fs) = if desc.options.aov {
            let mut options = CompileOptions::new().unwrap();
            options.add_macro_definition("GBUFFER_AOV", None);
            let vs = {
                compile_shader_file(
                    Path::new("src/shaders/gbuffer.vert"),
                    "gbuffer_hdr_aov.vert",
                    shaderc::ShaderKind::Vertex,
                    device,
                    Some(&options),
                )
                .unwrap()
            };
            let fs = {
                compile_shader_file(
                    Path::new("src/shaders/gbuffer.frag"),
                    "gbuffer_hdr_aov.frag",
                    shaderc::ShaderKind::Fragment,
                    device,
                    Some(&options),
                )
                .unwrap()
            };
            (vs, fs)
        } else {
            let vs = {
                compile_shader_file(
                    Path::new("src/shaders/gbuffer.vert"),
                    "gbuffer_hdr.vert",
                    shaderc::ShaderKind::Vertex,
                    device,
                    None,
                )
                .unwrap()
            };
            let fs = {
                compile_shader_file(
                    Path::new("src/shaders/gbuffer.frag"),
                    "gbuffer_hdr.frag",
                    shaderc::ShaderKind::Fragment,
                    device,
                    None,
                )
                .unwrap()
            };
            (vs, fs)
        };
        // let vs_src = std::fs::read_to_string("src/shaders/gbuffer.vert").unwrap();
        // let fs_src = std::fs::read_to_string("src/shaders/gbuffer.frag").unwrap();
        // let mut compiler = shaderc::Compiler::new().unwrap();
        // let size = Size::new(ctx.size.width, ctx.size.height);
        // let vs_spirv = compiler
        //     .compile_into_spirv(
        //         &vs_src,
        //         shaderc::ShaderKind::Vertex,
        //         "gbuffer.vert",
        //         "main",
        //         None,
        //     )
        //     .unwrap();
        // let fs_spirv = compiler
        //     .compile_into_spirv(
        //         &fs_src,
        //         shaderc::ShaderKind::Fragment,
        //         "gbuffer.frag",
        //         "main",
        //         None,
        //     )
        //     .unwrap();
        // let vs_data = wgpu::util::make_spirv(vs_spirv.as_binary_u8());
        // let fs_data = wgpu::util::make_spirv(fs_spirv.as_binary_u8());
        // let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        //     label: Some("GBufferPass Vertex Shader"),
        //     source: vs_data,
        //     flags: wgpu::ShaderFlags::default(),
        // });
        // let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        //     label: Some("GBufferPass Fragment Shader"),
        //     source: fs_data,
        //     flags: wgpu::ShaderFlags::default(),
        // });
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
        let gbuffer_formats = GBuffer::rt_formats(&desc.options);
        let render_targets: Vec<_> = (0..gbuffer_formats.len())
            .map(|i| wgpu::ColorTargetState {
                format: gbuffer_formats[i],
                blend: None,
                write_mask: wgpu::ColorWrite::ALL,
            })
            .collect();
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GBuferPass Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: "main",        // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &fs,
                entry_point: "main",
                targets: render_targets.as_slice(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
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
            ctx: ctx.clone(),
        }
    }
    fn record_command(
        &mut self,
        params: &Self::Params,
        _frame_ctx: &FrameContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> GBufferPassNode {
        self.camera_uniform.upload(
            &self.ctx.device_ctx,
            &[UniformViewProjection::new(
                &params.camera.build_view_projection_matrix(),
            )],
        );

        let attachment_descs: Vec<_> = (0..params.gbuffer.num_render_targets())
            .map(|i| wgpu::RenderPassColorAttachment {
                view: &params.gbuffer.render_targets[i as usize].view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })
            .collect();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: attachment_descs.as_slice(),
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &params.gbuffer.depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, &self.bind_group0, &[]);

        for m in &params.scene.meshes {
            render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
            render_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..m.num_indices, 0, 0..1); // 2.
        }
        GBufferPassNode {
            gbuffer: params.gbuffer.clone(),
        }
    }
}
