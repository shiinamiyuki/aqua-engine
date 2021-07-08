use std::{path::Path, sync::Arc};

use wgpu::util::DeviceExt;

use crate::render::{Buffer, BufferData, Camera, CubeMap, FrameContext, GBufferOptions, GPUScene, PointLight, PointLightData, RenderContext, RenderPass, Size, Texture, UniformViewProjection, Vertex, compile_shader_file};
use crate::render::GBuffer;
use super::{
    ComputePass, GBufferPass, GBufferPassParams, ShadowPass,
};

pub struct ShadowMapPass {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    light: Buffer<PointLightData>,
    ctx: Arc<RenderContext>,
    // seeds: wgpu::Buffer,
    // light_vp: Buffer<UniformViewProjection>,
}
pub struct ShadowMapPassDescriptor {
    pub ctx: Arc<RenderContext>,
    pub gbuffer_options:GBufferOptions,
}
pub struct ShadowMapPassNode{}
pub struct ShadowMapPassParams {
    pub scene: Arc<GPUScene>,
    pub light_idx: u32,
    pub cubemap: Arc<CubeMap>,
    pub gbuffer: GBuffer,
    pub color: Arc<Texture>,
}

impl ComputePass for ShadowMapPass {
    type Descriptor = ShadowMapPassDescriptor;
    fn create_pass(desc: &Self::Descriptor) -> Self {
        let ctx = &desc.ctx;
        let device = &ctx.device_ctx.device;
        let cs = compile_shader_file(
            Path::new("src/shaders/shadow_map.comp"),
            "shadow_map",
            shaderc::ShaderKind::Compute,
            &ctx.device_ctx.device,
            None
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
                        &GBuffer::bind_group_layout(&ctx.device_ctx, &desc.gbuffer_options),
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
        // let mut rng = rand::thread_rng();
        // let seeds_data: Vec<u32> = (0..(1920 * 1080)).map(|_| {
        //     rng.gen::<u32>()
        // }).collect();
        // let seeds = device.create_buffer_init(&wgpu::BufferInitDescriptor {
        //     label:Some("seeds"),
        //     contents:bytemuck::cast_slice(&seeds_data[..])
        // });
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
            ctx:ctx.clone(),
        }
    }
    type Params = ShadowMapPassParams;
    type Node = ShadowMapPassNode;
    fn record_command(
        &mut self,
        params: &Self::Params,
        encoder: &mut wgpu::CommandEncoder,
    ) -> ShadowMapPassNode {
        let ctx = &self.ctx;
        self.light.upload(
            &ctx.device_ctx,
            &[PointLightData::new(
                &params.scene.point_lights[params.light_idx as usize],
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
                ],
                layout: &self.bind_group_layout,
            });
        let bindgroup1 = params.gbuffer.create_bind_group(&ctx.device_ctx);
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Shadow Map pass"),
        });
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_push_constants(
            0,
            bytemuck::cast_slice(&[
                params.light_idx as i32,
                params.color.extent.width as i32,
                params.color.extent.height as i32,
            ]),
        );

        compute_pass.set_bind_group(0, &bindgroup0, &[]);
        compute_pass.set_bind_group(1, &bindgroup1, &[]);

        compute_pass.dispatch(
            (params.color.extent.width + 15) / 16,
            (params.color.extent.height + 15) / 16,
            1,
        );
        ShadowMapPassNode{}
    }
}
