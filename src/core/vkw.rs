use std::sync::Arc;

use ash::{
    prelude::VkResult,
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};

pub type PAllocationCallbacks = Arc<vk::AllocationCallbacks>;
pub struct Context {
    instance: ash::Instance,
    device: ash::Device,
    allocation_callbacks: *const vk::AllocationCallbacks,
}
impl Context {
    pub fn new(
        instance: ash::Instance,
        device: ash::Device,
        allocation_callbacks: Option<&vk::AllocationCallbacks>,
    ) -> Self {
        Self {
            instance,
            device,
            allocation_callbacks: allocation_callbacks
                .map_or(std::ptr::null(), |x| x as *const vk::AllocationCallbacks),
        }
    }
    pub fn device(&self) -> &ash::Device {
        &self.device
    }
    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_device(self.allocation_callbacks.as_ref());
            self.instance
                .destroy_instance(self.allocation_callbacks.as_ref());
        }
    }
}

pub struct Device {
    context: Arc<Context>,
}
impl Device {
    pub fn from_context(ctx: Context) -> Self {
        Self {
            context: Arc::new(ctx),
        }
    }
}
macro_rules! define_wrapper_struct {
    ($name:ident, $handle:ty) => {
        pub struct $name {
            handle: $handle,
            context: Arc<Context>,
        }
        impl $name {
            pub fn handle(&self) -> $handle {
                self.handle
            }
            pub fn context(&self) -> &Arc<Context> {
                &self.context
            }
        }
    };
}
macro_rules! define_drop {
    ($name:ident,$func:ident) => {
        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    self.context
                        .device
                        .$func(self.handle, self.context.allocation_callbacks.as_ref());
                }
            }
        }
    };
}
macro_rules! define_wrapper {
    ($name:ident, $handle:ty) => {
        define_wrapper_struct!($name, $handle);
    };
    ($name:ident, $handle:ty, create=[$( ($create_info:ty, $func:ident) ),*]) => {
        define_wrapper_struct!($name, $handle);
        $(define_create!($name, $create_info, $func);)+
    };
    ($name:ident, $handle:ty, create=[$( ($create_info:ty, $func:ident) ),*], drop=$drop:ident) => {
        define_wrapper_struct!($name, $handle);
        $(define_create!($name, $create_info, $func);)+
        define_drop!($name,$drop);
    };
}
macro_rules! define_create {
    ($name:ident, $create_info:ty, $func:ident) => {
        impl Device {
            pub fn $func(&self, create_info: &$create_info) -> VkResult<$name> {
                unsafe {
                    Ok($name {
                        handle: self
                            .context
                            .device
                            .$func(create_info, self.context.allocation_callbacks.as_ref())?,
                        context: self.context.clone(),
                    })
                }
            }
        }
    };
}

impl Device {
    pub fn handle(&self) -> &ash::Device {
        self.context.device()
    }
    pub fn context(&self) -> &Arc<Context> {
        &self.context
    }
}
define_wrapper!(
    Image,
    vk::Image,
    create = [(vk::ImageCreateInfo, create_image)],
    drop = destroy_image
);
define_wrapper!(
    ImageView,
    vk::ImageView,
    create = [(vk::ImageViewCreateInfo, create_image_view)],
    drop = destroy_image_view
);
define_wrapper!(
    Semaphore,
    vk::Semaphore,
    create = [(vk::SemaphoreCreateInfo, create_semaphore)],
    drop = destroy_semaphore
);
define_wrapper!(
    Fence,
    vk::Fence,
    create = [(vk::FenceCreateInfo, create_fence)],
    drop = destroy_fence
);
define_wrapper!(
    Buffer,
    vk::Buffer,
    create = [(vk::BufferCreateInfo, create_buffer)],
    drop = destroy_buffer
);
define_wrapper!(
    CommandPool,
    vk::CommandPool,
    create = [(vk::CommandPoolCreateInfo, create_command_pool)],
    drop = destroy_command_pool
);
define_wrapper!(Pipeline, vk::Pipeline);
define_wrapper!(
    PipelineLayout,
    vk::PipelineLayout,
    create = [(vk::PipelineLayoutCreateInfo, create_pipeline_layout)],
    drop = destroy_pipeline_layout
);
define_wrapper!(CommandBuffer, vk::CommandBuffer);
define_wrapper!(PipelineCache, vk::PipelineCache);

impl CommandBuffer {
    pub fn reset(&self, flags: vk::CommandBufferResetFlags) -> VkResult<()> {
        unsafe { self.context.device.reset_command_buffer(self.handle, flags) }
    }
}

impl CommandPool {
    pub fn reset(&self, flags: vk::CommandPoolResetFlags) -> VkResult<()> {
        unsafe { self.context.device.reset_command_pool(self.handle, flags) }
    }
}
pub struct SwapChainContext {
    swapchain_loader: ash::extensions::khr::Swapchain,
}
pub struct Swapchain {
    handle: vk::SwapchainKHR,
    sc_context: Arc<SwapChainContext>,
    context: Arc<Context>,
}
impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.sc_context
                .swapchain_loader
                .destroy_swapchain(self.handle, self.context.allocation_callbacks.as_ref());
        }
    }
}
impl Device {
    pub fn create_compute_pipline(
        &self,
        pipeline_cache: Option<&PipelineCache>,
        create_info: &vk::ComputePipelineCreateInfo,
    ) -> VkResult<Pipeline> {
        unsafe {
            match self.context.device.create_compute_pipelines(
                pipeline_cache.map_or(vk::PipelineCache::null(), |x| x.handle),
                &[*create_info],
                self.context.allocation_callbacks.as_ref(),
            ) {
                Ok(pipelines) => Ok(Pipeline {
                    context: self.context.clone(),
                    handle: pipelines[0],
                }),
                Err(e) => Err(e.1),
            }
        }
    }
    pub fn create_graphpics_pipline(
        &self,
        pipeline_cache: Option<&PipelineCache>,
        create_info: &vk::GraphicsPipelineCreateInfo,
    ) -> VkResult<Pipeline> {
        unsafe {
            match self.context.device.create_graphics_pipelines(
                pipeline_cache.map_or(vk::PipelineCache::null(), |x| x.handle),
                &[*create_info],
                self.context.allocation_callbacks.as_ref(),
            ) {
                Ok(pipelines) => Ok(Pipeline {
                    context: self.context.clone(),
                    handle: pipelines[0],
                }),
                Err(e) => Err(e.1),
            }
        }
    }
    pub fn create_swapchain(
        &self,
        create_info: &vk::SwapchainCreateInfoKHR,
    ) -> VkResult<Swapchain> {
        let loader =
            ash::extensions::khr::Swapchain::new(self.context.instance(), self.context.device());
        unsafe {
            let sc =
                loader.create_swapchain(create_info, self.context.allocation_callbacks.as_ref())?;
            Ok(Swapchain {
                context: self.context.clone(),
                sc_context: Arc::new(SwapChainContext {
                    swapchain_loader: loader,
                }),
                handle: sc,
            })
        }
    }
}

pub struct CommandEncoder<'a> {
    command_buffer: &'a CommandBuffer,
    context: Arc<Context>,
}
impl CommandBuffer {
    pub fn begin_command_buffer<'a>(
        &'a self,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> VkResult<CommandEncoder<'a>> {
        unsafe {
            self.context
                .device
                .begin_command_buffer(self.handle, begin_info)?;
            Ok(CommandEncoder {
                command_buffer: self,
                context: self.context.clone(),
            })
        }
    }
}
impl<'a> Drop for CommandEncoder<'a> {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .end_command_buffer(self.command_buffer.handle)
                .expect("fail to submit command buffer");
        }
    }
}
impl<'a> CommandEncoder<'a> {
    pub fn begin_render_pass(
        &'a self,
        create_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) -> RenderPass<'a> {
        unsafe {
            self.context.device.cmd_begin_render_pass(
                self.command_buffer.handle,
                create_info,
                contents,
            );
            RenderPass { encoder: self }
        }
    }
}
pub struct RenderPass<'a> {
    encoder: &'a CommandEncoder<'a>,
}
impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        unsafe {
            self.encoder
                .context
                .device
                .cmd_end_render_pass(self.encoder.command_buffer.handle);
        }
    }
}
impl<'a> RenderPass<'a> {
    pub fn bind_pipeline(&self, pipeline_bind_point: vk::PipelineBindPoint, pipeline: &Pipeline) {
        unsafe {
            self.encoder
                .command_buffer
                .context
                .device
                .cmd_bind_pipeline(
                    self.encoder.command_buffer.handle,
                    pipeline_bind_point,
                    pipeline.handle,
                );
        }
    }
    pub fn set_viewport(&self, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe {
            self.encoder.command_buffer.context.device.cmd_set_viewport(
                self.encoder.command_buffer.handle,
                first_viewport,
                viewports,
            )
        }
    }
    pub fn bind_index_buffer(&self, buffer: &Buffer, offset: u64, index_type: vk::IndexType) {
        unsafe {
            self.encoder
                .command_buffer
                .context
                .device
                .cmd_bind_index_buffer(
                    self.encoder.command_buffer.handle,
                    buffer.handle,
                    offset,
                    index_type,
                )
        }
    }
    pub fn bind_vertex_buffer(&self, binding: u32, buffer: &Buffer, offset: u64) {
        unsafe {
            self.encoder
                .command_buffer
                .context
                .device
                .cmd_bind_vertex_buffers(
                    self.encoder.command_buffer.handle,
                    binding,
                    &[buffer.handle],
                    &[offset],
                )
        }
    }
    pub fn bind_descriptor_set(
        &self,
        pipeline_bind_point: vk::PipelineBindPoint,
        layout: &PipelineLayout,
        binding: u32,
        descriptor_set: &vk::DescriptorSet,
    ) {
        unsafe {
            self.encoder
                .command_buffer
                .context
                .device
                .cmd_bind_descriptor_sets(
                    self.encoder.command_buffer.handle,
                    pipeline_bind_point,
                    layout,
                    first_set,
                    descriptor_sets,
                    dynamic_offsets,
                )
        }
    }
    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.encoder.command_buffer.context.device.cmd_draw_indexed(
                self.encoder.command_buffer.handle,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }
}
