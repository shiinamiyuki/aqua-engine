use std::sync::Arc;

use ash::{
    prelude::VkResult,
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
// vkw for vk wrapper
macro_rules! impl_wrapper_deref {
    ($wrapper:ty,$handle:ty) => {
        impl std::ops::Deref for $wrapper {
            type Target = $handle;
            fn deref(&self) -> &Self::Target {
                &self.handle
            }
        }
    };
}
macro_rules! impl_wrapper_deref_drop {
    ($wrapper:ty,$handle:ty, $drop:ident) => {
        impl_wrapper_deref!($wrapper, $handle);
        impl Drop for $wrapper {
            fn drop(&mut self) {
                unsafe {
                    let cb = if let Some(cb) = &self.allocation_callbacks {
                        Some(cb.as_ref())
                    } else {
                        None
                    };
                    self.device.$drop(self.handle, cb);
                }
            }
        }
    };
}
macro_rules! impl_wrapper_new {
    ($wrapper:ty,$create_info:ty,$new:ident) => {
        impl $wrapper {
            pub fn new(
                device: &Device,
                create_info: &$create_info,
                allocation_callbacks: Option<PAllocationCallbacks>,
            ) -> VkResult<Self> {
                unsafe {
                    let cb = if let Some(cb) = &allocation_callbacks {
                        Some(cb.as_ref())
                    } else {
                        None
                    };
                    let handle = device.$new(&create_info, cb);
                    match handle {
                        Ok(handle) => Ok(Self {
                            handle,
                            device: device.handle.clone(),
                            allocation_callbacks,
                        }),
                        Err(e) => Err(e),
                    }
                }
            }
        }
    };
}
macro_rules! impl_wrapper {
    ($wrapper:ty,$handle:ty, drop=$drop:ident) => {
        impl_wrapper_deref_drop!($wrapper, $handle, $drop);
    };
    ($wrapper:ty,$handle:ty, drop=$drop:ident,new={$create_info:ty, $new:ident}) => {
        impl_wrapper_deref_drop!($wrapper, $handle, $drop);
        impl_wrapper_new!($wrapper, $create_info, $new);
    };
}
pub type PAllocationCallbacks = Arc<vk::AllocationCallbacks>;
pub struct Device {
    pub handle: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl std::ops::Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
macro_rules! get_cb {
    ($cb:expr) => {{
        if let Some(cb) = &$cb {
            Some(cb.as_ref())
        } else {
            None
        }
    }};
}
impl Device {
    pub fn new(
        instance: &ash::Instance,
        pdevice: vk::PhysicalDevice,
        create_info: &vk::DeviceCreateInfo,
        allocation_callbacks: Option<PAllocationCallbacks>,
    ) -> VkResult<Self> {
        unsafe {
            let cb = get_cb!(allocation_callbacks);
            let device = instance.create_device(pdevice, &create_info, cb);
            match device {
                Ok(device) => Ok(Self {
                    handle: device,
                    allocation_callbacks,
                }),
                Err(e) => Err(e),
            }
        }
    }
}
pub struct Fence {
    pub handle: vk::Fence,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    Fence, vk::Fence,
    drop=destroy_fence,
    new={
        vk::FenceCreateInfo,
        create_fence
    }
}

pub struct Semaphore {
    pub handle: vk::Semaphore,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    Semaphore, vk::Semaphore,
    drop=destroy_semaphore,
    new={
        vk::SemaphoreCreateInfo,
        create_semaphore
    }
}

pub struct Image {
    pub handle: vk::Image,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    Image, vk::Image,
    drop=destroy_image,
    new={
        vk::ImageCreateInfo,
        create_image
    }
}

pub struct ImageView {
    pub handle: vk::ImageView,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    ImageView, vk::ImageView,
    drop=destroy_image_view,
    new={
        vk::ImageViewCreateInfo,
        create_image_view
    }
}

pub struct SwapChainLoader {
    pub handle: ash::extensions::khr::Swapchain,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}

impl_wrapper_deref!(SwapChainLoader, ash::extensions::khr::Swapchain);

pub struct SwapChain {
    pub handle: vk::SwapchainKHR,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
    pub swapchain: ash::extensions::khr::Swapchain,
}
impl_wrapper_deref!(SwapChain, vk::SwapchainKHR);

impl Drop for SwapChain {
    fn drop(&mut self) {
        unsafe {
            let cb = get_cb!(self.allocation_callbacks);
            self.swapchain.destroy_swapchain(self.handle, cb)
        }
    }
}

pub struct Buffer {
    pub handle: vk::Buffer,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    Buffer,
    vk::Buffer,
    drop=destroy_buffer,
    new={
        vk::BufferCreateInfo,
        create_buffer
    }
}

pub struct ShaderModule {
    pub handle: vk::ShaderModule,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    ShaderModule, vk::ShaderModule,
    drop=destroy_shader_module,
    new={
        vk::ShaderModuleCreateInfo,
        create_shader_module
    }
}
pub struct PipelineCache {
    pub handle: vk::PipelineCache,
    pub device: Option<ash::Device>,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper_deref!(PipelineCache, vk::PipelineCache);
impl Drop for PipelineCache {
    fn drop(&mut self) {
        if let Some(device) = self.device {
            unsafe {
                device.destroy_pipeline_cache(self.handle, get_cb!(self.allocation_callbacks));
            }
        }
    }
}
impl PipelineCache {
    fn null() -> Self {
        Self {
            handle: vk::PipelineCache::null(),
            device: None,
            allocation_callbacks: None,
        }
    }
}
pub struct PipelineLayout {
    pub handle: vk::PipelineLayout,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    PipelineLayout, vk::PipelineLayout,
    drop=destroy_pipeline_layout,
    new={
        vk::PipelineLayoutCreateInfo,
        create_pipeline_layout
    }
}
pub struct Pipeline {
    pub handle: vk::Pipeline,
    pub device: ash::Device,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}
impl_wrapper! {
    Pipeline, vk::Pipeline,
    drop=destroy_pipeline
}
impl Pipeline {
    pub fn new_compute_pipelines(
        device: &Device,
        pipeline_cache: &PipelineCache,
        create_infos: &[vk::ComputePipelineCreateInfo],
        allocation_callbacks: Option<PAllocationCallbacks>,
    ) -> VkResult<Vec<Self>> {
        let cb = get_cb!(allocation_callbacks);
        unsafe {
            match device.create_compute_pipelines(pipeline_cache.handle, create_infos, cb) {
                Ok(pipelines) => Ok(pipelines
                    .iter()
                    .map(|x| -> Pipeline {
                        Pipeline {
                            handle: x.clone(),
                            device: device.handle,
                            allocation_callbacks: allocation_callbacks.clone(),
                        }
                    })
                    .collect()),
                Err(e) => Err(e),
            }
        }
    }
    pub fn new_graphics_piplines(
        device: &Device,
        pipeline_cache: &PipelineCache,
        create_infos: &[vk::GraphicsPipelineCreateInfo],
        allocation_callbacks: Option<PAllocationCallbacks>,
    ) -> VkResult<Vec<Self>> {
        let cb = get_cb!(allocation_callbacks);
        unsafe {
            match device.create_graphics_pipelines(pipeline_cache.handle, create_infos, cb) {
                Ok(pipelines) => Ok(pipelines
                    .iter()
                    .map(|x| -> Pipeline {
                        Pipeline {
                            handle: x.clone(),
                            device: device.handle,
                            allocation_callbacks: allocation_callbacks.clone(),
                        }
                    })
                    .collect()),
                Err(e) => Err(e.1),
            }
        }
    }
}
