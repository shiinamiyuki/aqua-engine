use std::sync::Arc;

use super::{Buffer, BufferData, FrameContext, PointLight, Texture, UniformViewProjection, Vertex};
use super::{Camera, GPUMesh, Size};
use super::{PointLightData, RenderContext};
use crate::glm;
use env_logger::fmt::Color;

pub struct ColorAttachment<'a> {
    pub view: &'a wgpu::TextureView,
}
impl <'a> ColorAttachment<'a> {
    pub fn get_descriptor(&self) -> wgpu::RenderPassColorAttachmentDescriptor {
        wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &self.view,
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
        }
    }
}
// pub struct RenderInput<'a, 'b> {
//     pub attachements: Vec<ColorAttachment<'b>>,
//     pub meshes: &'a [GPUMesh],
// }
pub trait RenderPass {
    type Input;
    fn record_command(
        &mut self,
        size: Size,
        ctx: &mut RenderContext,
        frame_ctx: &mut FrameContext,
        camera: &dyn Camera,
        input: &Self::Input,
    ) -> wgpu::CommandBuffer;
}
pub struct SimpleRenderPassInput {
    pub meshes: Vec<Arc<GPUMesh>>,
}

pub struct SimpleRenderPass {
    pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    camera_uniform: Buffer<UniformViewProjection>,
    size: Size,
    light: PointLight,
    light_buffer: Buffer<PointLightData>,
}
impl RenderPass for SimpleRenderPass {
    type Input = SimpleRenderPassInput;
    fn record_command(
        &mut self,
        size: Size,
        ctx: &mut RenderContext,
        frame_ctx: &mut FrameContext,
        camera: &dyn Camera,
        input: &Self::Input,
    ) -> wgpu::CommandBuffer {
        if self.size != size {
            self.size = size;
            self.depth_texture = Texture::create_depth_texture_from_sc(
                &ctx.device_ctx.device,
                &ctx.sc_desc,
                "depth texture",
            );
        }
        self.camera_uniform.upload(
            &ctx.device_ctx,
            &[UniformViewProjection::new(
                &camera.build_view_projection_matrix(),
            )],
        );
        self.light_buffer
            .upload(&ctx.device_ctx, &[PointLightData::new(&self.light)]);
        let mut encoder =
            ctx.device_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            // 1.
            let depth_view = &self.depth_texture.view;
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_pipeline(&self.pipeline); // 2.
                                                      // render_pass.draw(0..3, 0..1); // 3.
                                                      // render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(0, &self.camera_uniform.bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_buffer.bind_group, &[]);
            for m in &input.meshes {
                render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                render_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..m.num_indices, 0, 0..1); // 2.
            }
        }
        encoder.finish()
    }
}

impl SimpleRenderPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let device = &ctx.device_ctx.device;
        let sc_desc = &ctx.sc_desc;
        let vs_src = std::fs::read_to_string("src/shaders/shader.vert").unwrap();
        let fs_src = std::fs::read_to_string("src/shaders/shader.frag").unwrap();
        let mut compiler = shaderc::Compiler::new().unwrap();
        let size = Size(ctx.size.width, ctx.size.height);
        let vs_spirv = compiler
            .compile_into_spirv(
                &vs_src,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                &fs_src,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();
        let vs_data = wgpu::util::make_spirv(vs_spirv.as_binary_u8());
        let fs_data = wgpu::util::make_spirv(fs_spirv.as_binary_u8());
        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: vs_data,
            flags: wgpu::ShaderFlags::default(),
        });
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: fs_data,
            flags: wgpu::ShaderFlags::default(),
        });
        let camera_uniform = Buffer::<UniformViewProjection>::new_uniform_buffer(
            &ctx.device_ctx,
            0,
            wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
            &[UniformViewProjection::default()],
            None,
        );
        let point_light = PointLight {
            position: glm::vec3(0.0, 1.2, 2.0),
            emission: glm::vec3(1.0, 1.0, 1.0),
        };
        let light_buffer = Buffer::<PointLightData>::new_uniform_buffer(
            &ctx.device_ctx,
            0,
            wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
            &[PointLightData::new(&point_light)],
            Some("light"),
        );
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_uniform.layout, &light_buffer.layout],
                push_constant_ranges: &[],
            });
        let depth_texture =
            Texture::create_depth_texture_from_sc(&device, &sc_desc, "depth_texture");
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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
                targets: &[wgpu::ColorTargetState {
                    // 4.
                    format: sc_desc.format,
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
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
        });

        Self {
            pipeline: render_pipeline,
            depth_texture,
            camera_uniform,
            size,
            light: point_light,
            light_buffer,
        }
    }
}
