use std::{path::Path, sync::Arc};

use render::{opengl_to_wgpu_matrix, LookAtCamera, Perspective};
use wgpu::util::DeviceExt;

use crate::glm;
use crate::render::{
    self, compile_shader_file, Buffer, BufferData, Camera, CubeMap, FrameContext, GPUScene,
    RenderContext, RenderPass, Size, Texture, UniformViewProjection, Vertex, ViewProjection,
};

pub struct ShadowPassParams {
    pub scene: Arc<GPUScene>,
    pub light_idx: u32,
    pub cubemap: Arc<CubeMap>,
}
pub struct ShadowPass {
    pipeline: wgpu::RenderPipeline,
    light_vp: Buffer<UniformViewProjection>,
    bindgroup: wgpu::BindGroup,
    depth: Texture,
    cubemap_res: u32,
    ctx: Arc<RenderContext>,
}
pub struct ShadowPassDescriptor {
    pub ctx: Arc<RenderContext>,
    pub cubemap_res: u32,
}
pub struct ShadowPassNode {
    pub cubemap: Arc<CubeMap>,
    pub cubemap_res: u32,
}
impl RenderPass for ShadowPass {
    type Descriptor = ShadowPassDescriptor;
    fn create_pass(desc: &Self::Descriptor) -> Self {
        let ctx = &desc.ctx;
        let device = &ctx.device_ctx.device;
        let fs = compile_shader_file(
            Path::new("src/shaders/shadow.frag"),
            shaderc::ShaderKind::Fragment,
            &ctx.device_ctx.device,
        )
        .unwrap();
        let vs = compile_shader_file(
            Path::new("src/shaders/shadow.vert"),
            shaderc::ShaderKind::Vertex,
            &ctx.device_ctx.device,
        )
        .unwrap();
        let light_vp = Buffer::new_uniform_buffer(
            &ctx.device_ctx,
            &[UniformViewProjection::default(); 6],
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
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStage::VERTEX,
                range: 0..4,
            }],
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
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
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
        let cubemap_res = desc.cubemap_res;
        let depth = Texture::create_depth_texture_with_size(
            device,
            &Size(cubemap_res, cubemap_res),
            "shadow.depth",
        );

        Self {
            light_vp,
            pipeline,
            bindgroup,
            depth,
            cubemap_res,
            ctx: ctx.clone(),
        }
    }
    type Params = ShadowPassParams;
    type Node = ShadowPassNode;
    fn record_command(
        &mut self,
        params: &Self::Params,
        _frame_ctx: &FrameContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> ShadowPassNode {
        let mut vp = vec![];
        for face in 0..6 {
            let proj = glm::perspective(1.0f32, std::f32::consts::PI * 0.5, 0.01, 100.0);
            let dir = {
                match face {
                    0 => glm::vec3(-1.0, 0.0, 0.0),
                    1 => glm::vec3(1.0, 0.0, 0.0),
                    2 => glm::vec3(0.0, 1.0, 0.0),
                    3 => glm::vec3(0.0, -1.0, 0.0),
                    4 => glm::vec3(0.0, 0.0, 1.0),
                    5 => glm::vec3(0.0, 0.0, -1.0),
                    _ => unreachable!(),
                }
            };
            let up = {
                match face {
                    2 => glm::vec3(0.0, 0.0, -1.0),
                    3 => glm::vec3(0.0, 0.0, 1.0),
                    _ => glm::vec3(0.0, 1.0, 0.0),
                }
            };
            let eye = params.scene.point_lights[params.light_idx as usize].position;
            let view = glm::look_at(&eye, &(eye + dir), &up);
            vp.push(UniformViewProjection::new(&ViewProjection(
                view,
                opengl_to_wgpu_matrix() * proj,
            )));
        }
        self.light_vp.upload(&self.ctx.device_ctx, &vp[..]);

        for face in 0..6 {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow.pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &params.cubemap.face_views[face as usize],
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_push_constants(
                wgpu::ShaderStage::VERTEX,
                0,
                bytemuck::cast_slice(&[face as i32]),
            );
            render_pass.set_bind_group(0, &self.bindgroup, &[]);
            params.scene.draw(&mut render_pass);
        }
        ShadowPassNode {
            cubemap: params.cubemap.clone(),
            cubemap_res: self.cubemap_res,
        }
    }
}
