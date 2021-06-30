use std::sync::Arc;

use super::{
    Camera, ComputePass, CubeMap, FrameContext, GBuffer, GBufferOptions, GBufferPass,
    GBufferPassDescriptor, GBufferPassParams, GPUScene, PostProcessPass, PostProcessPassDescriptor,
    PostProcessPassParams, RenderContext, RenderPass, SSGIPass, SSGIPassDescriptor, SSGIPassParams,
    ShadowMapPass, ShadowMapPassDescriptor, ShadowPass, ShadowPassDescriptor, ShadowPassParams,
    Size, Texture,
};

pub trait RenderPipeline {
    type Descriptor;
    fn create_pipeline(desc: &Self::Descriptor) -> Self;
    type Params;
    fn record_command(
        &mut self,
        params: &Self::Params,
        frame_ctx: &FrameContext,
        encoder: &mut wgpu::CommandEncoder,
    );
}

pub struct DeferredShadingPipelineDescriptor {
    pub ctx: Arc<RenderContext>,
    pub gbuffer_options: GBufferOptions,
}

pub struct DeferredShadingPipeline {
    shadow_pass: ShadowPass,
    shadow_map_pass: ShadowMapPass,
    gbuffer_pass: GBufferPass,
    ssgi_pass: SSGIPass,
    post_process_pass: PostProcessPass,
    gbuffer: Arc<GBuffer>,
    shadow_cube_map: Arc<CubeMap>,
    color_buffer: Arc<Texture>,
    ctx: Arc<RenderContext>,
}

#[derive(Clone)]
pub struct DeferredShadingParams {
    pub scene: Arc<GPUScene>,
    pub camera: Camera,
}

impl RenderPipeline for DeferredShadingPipeline {
    type Descriptor = DeferredShadingPipelineDescriptor;
    fn create_pipeline(desc: &Self::Descriptor) -> Self {
        let ctx = &desc.ctx;
        let gbuffer = GBuffer::new(
            &ctx.device_ctx,
            &Size::new(ctx.size.width, ctx.size.height),
            &desc.gbuffer_options,
        );
        let cubemap_res = 512;
        let shadow_cube_map = Arc::new(CubeMap::create_cubemap(
            &ctx.device_ctx.device,
            cubemap_res,
            wgpu::TextureFormat::R32Float,
            "omni-shadow",
            true,
        ));
        let color_buffer = Arc::new(Texture::create_color_attachment(
            &ctx.device_ctx.device,
            &ctx.size,
            wgpu::TextureFormat::Rgba32Float,
            "deferred.color",
        ));
        Self {
            gbuffer: Arc::new(gbuffer),
            shadow_cube_map,
            shadow_pass: ShadowPass::create_pass(&ShadowPassDescriptor {
                ctx: ctx.clone(),
                cubemap_res,
            }),
            shadow_map_pass: ShadowMapPass::create_pass(&ShadowMapPassDescriptor {
                ctx: ctx.clone(),
                gbuffer_options: desc.gbuffer_options,
            }),
            gbuffer_pass: GBufferPass::create_pass(&GBufferPassDescriptor {
                ctx: ctx.clone(),
                options: desc.gbuffer_options,
            }),
            post_process_pass: PostProcessPass::create_pass(&PostProcessPassDescriptor {
                ctx: ctx.clone(),
            }),
            ssgi_pass: SSGIPass::create_pass(&SSGIPassDescriptor {
                ctx: ctx.clone(),
                gbuffer_options: desc.gbuffer_options,
            }),
            ctx: ctx.clone(),
            color_buffer,
        }
    }
    type Params = DeferredShadingParams;
    fn record_command(
        &mut self,
        params: &Self::Params,
        frame_ctx: &FrameContext,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let camera = &params.camera;
        {
            let params = ShadowPassParams {
                scene: params.scene.clone(),
                light_idx: 0,
                cubemap: self.shadow_cube_map.clone(),
            };
            self.shadow_pass.record_command(&params, frame_ctx, encoder);
        }
        {
            let params = GBufferPassParams {
                scene: params.scene.clone(),
                gbuffer: self.gbuffer.clone(),
                camera: params.camera.clone(),
            };
            {
                self.gbuffer_pass
                    .record_command(&params, frame_ctx, encoder);
            }
        }
        // {
        //     let params = ShadowMapPassParams {
        //         scene: params.scene.clone(),
        //         light_idx: 0,
        //         cubemap: self.shadow_cube_map.clone(),
        //         gbuffer: self.gbuffer.clone(),
        //         color: self.color_buffer.clone(),
        //     };
        //     self.shadow_map_pass
        //         .record_command(ctx, frame_ctx, camera, &params, encoder)
        // }
        {
            let params = SSGIPassParams {
                scene: params.scene.clone(),
                light_idx: 0,
                cubemap: self.shadow_cube_map.clone(),
                gbuffer: self.gbuffer.clone(),
                color: self.color_buffer.clone(),
                view_dir: camera.dir(),
                eye_pos: camera.pos(),
                vp: camera.build_view_projection_matrix(),
            };
            self.ssgi_pass.record_command(&params, frame_ctx, encoder);
        }
        {
            let params = PostProcessPassParams {
                color_buf: self.color_buffer.clone(),
            };
            self.post_process_pass
                .record_command(&params, frame_ctx, encoder);
        }
    }
}
