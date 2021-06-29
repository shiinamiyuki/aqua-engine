use std::{num::NonZeroU32, path::Path, sync::Arc};

use crate::glm;
use crate::render::DepthQuadTree;
use crate::render::GBuffer;
use crate::render::SSRTBindGroup;
use crate::render::SSRTUniform;
use crate::{
    render::{
        compile_shader_file, Buffer, BufferData, Camera, ComputePass, CubeMap, DeviceContext,
        FrameContext, GPUScene, PointLight, PointLightData, RenderContext, RenderPass, Size,
        Texture, UniformViewProjection, Vertex, ViewProjection,
    },
    util,
};
use nalgebra_glm::Vec3;
use rand::Rng;
use wgpu::util::DeviceExt;
pub struct SSGIPass {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    light: Buffer<PointLightData>,
    seeds: wgpu::Buffer,
    camera_uniform: Buffer<UniformViewProjection>,
    camera_bindgroup: wgpu::BindGroup,
    depth_quad_tree: DepthQuadTree,
    ctx: Arc<RenderContext>,
    ssrt_bindgroup: SSRTBindGroup,
    // light_vp: Buffer<UniformViewProjection>,
}

pub struct SSGIPassParams {
    pub scene: Arc<GPUScene>,
    pub light_idx: u32,
    pub cubemap: Arc<CubeMap>,
    pub gbuffer: Arc<GBuffer>,
    pub color: Arc<Texture>,
    pub view_dir: Vec3,
    pub eye_pos: Vec3,
    pub vp: ViewProjection,
}
pub struct SSGIPassDescriptor {
    pub ctx: Arc<RenderContext>,
}

pub struct SSGIPassNode {}

impl RenderPass for SSGIPass {
    type Descriptor = SSGIPassDescriptor;
    fn create_pass(desc: &Self::Descriptor) -> Self {
        let ctx = &desc.ctx;
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
        let zquad_level = 5;

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
        let ssrt_bindgroup = SSRTBindGroup::new(&ctx.device_ctx);
        let depth_quad_tree = DepthQuadTree::new(ctx, zquad_level);
        let pipeline_layout =
            ctx.device_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ssgi.pipeline.layout"),
                    bind_group_layouts: &[
                        &bind_group_layout,
                        &GBuffer::bind_group_layout(&ctx.device_ctx, true),
                        &ssrt_bindgroup.layout,
                        &depth_quad_tree.zquad_bind_group_layout,
                        &camera_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
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
            depth_quad_tree,
            ssrt_bindgroup,
            ctx: ctx.clone(),
        }
    }
    type Params = SSGIPassParams;
    type Node = SSGIPassNode;
    fn record_command(
        &mut self,
        params: &Self::Params,
        _frame_ctx: &FrameContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> SSGIPassNode {
        let ctx = &self.ctx;
        self.depth_quad_tree
            .record_command(&ctx.device_ctx, &*params.gbuffer, encoder);
        self.light.upload(
            &ctx.device_ctx,
            &[PointLightData::new(
                &params.scene.point_lights[params.light_idx as usize],
            )],
        );
        self.ssrt_bindgroup.buffer.upload(
            &ctx.device_ctx,
            &[BufferData::new(&SSRTUniform {
                image_width: ctx.size.width,
                image_height: ctx.size.height,
                lod_width: self.depth_quad_tree.width * 2,
                lod_height: self.depth_quad_tree.height * 2,
                max_level: self.depth_quad_tree.level,
                near: 0.0,
                view_dir: params.view_dir,
                eye_pos: params.eye_pos,
            })],
        );
        let bindgroup0 = ctx
            .device_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ssgi.bindgroup0"),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&params.color.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&params.cubemap.view),
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
        let bindgroup1 = params.gbuffer.create_bind_group(&ctx.device_ctx);
        // // let bindgroup3
        // let bindgroup3 = ctx.device_ctx.device.create_bind_group(&wgpu::BindGroupDescriptor{
        //     label:Some("ssgi.zquad.trace.bindgroup"),
        //     layout:&self.zquad_bind_group_layout,
        //     entries:&[

        //     ]
        // });
        self.camera_uniform
            .upload(&ctx.device_ctx, &[UniformViewProjection::new(&params.vp)]);
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("SSGI pass"),
        });
        compute_pass.set_pipeline(&self.pipeline);

        // println!("{}", params.view_dir);
        compute_pass.set_bind_group(0, &bindgroup0, &[]);
        compute_pass.set_bind_group(1, &bindgroup1, &[]);
        compute_pass.set_bind_group(2, &self.ssrt_bindgroup.bindgroup, &[]);
        compute_pass.set_bind_group(3, &self.depth_quad_tree.zquad_bind_group, &[]);
        compute_pass.set_bind_group(4, &self.camera_bindgroup, &[]);
        compute_pass.dispatch(
            (params.color.extent.width + 15) / 16,
            (params.color.extent.height + 15) / 16,
            1,
        );
        SSGIPassNode {}
    }
}
