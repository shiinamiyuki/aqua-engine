use std::sync::Arc;

use ash::{
    prelude::VkResult,
    version::{DeviceV1_0, InstanceV1_0},
    vk::{self, AllocationCallbacks},
};

// vkw for vk wrapper
macro_rules! impl_wrapper_deref {
    ($wrapper:ty,$inner:ty) => {
        impl std::ops::Deref for $wrapper {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };
}
macro_rules! impl_wrapper_deref_drop {
    ($wrapper:ty,$inner:ty, $drop:ident) => {
        impl_wrapper_deref!($wrapper, $inner);
        impl Drop for $wrapper {
            fn drop(&mut self) {
                unsafe {
                    let cb = if let Some(cb) = &self.allocation_callbacks {
                        Some(cb.as_ref())
                    } else {
                        None
                    };
                    self.device.$drop(self.inner, cb);
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
                    let inner = device.$new(&create_info, cb);
                    match inner {
                        Ok(inner) => Ok(Self {
                            inner,
                            device: device.inner.clone(),
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
    ($wrapper:ty,$inner:ty, drop=$drop:ident) => {
        impl_wrapper_deref_drop!($wrapper, $inner, $drop);
    };
    ($wrapper:ty,$inner:ty, drop=$drop:ident,new={$create_info:ty, $new:ident}) => {
        impl_wrapper_deref_drop!($wrapper, $inner, $drop);
        impl_wrapper_new!($wrapper, $create_info, $new);
    };
}
pub type PAllocationCallbacks = Arc<vk::AllocationCallbacks>;
pub struct Device {
    inner: ash::Device,
    allocation_callbacks: Option<PAllocationCallbacks>,
}
impl std::ops::Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.inner
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
                    inner: device,
                    allocation_callbacks,
                }),
                Err(e) => Err(e),
            }
        }
    }
}
pub struct Fence {
    pub inner: vk::Fence,
    pub device: ash::Device,
    allocation_callbacks: Option<PAllocationCallbacks>,
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
    pub inner: vk::Semaphore,
    pub device: ash::Device,
    allocation_callbacks: Option<PAllocationCallbacks>,
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
    pub inner: vk::Image,
    pub device: ash::Device,
    allocation_callbacks: Option<PAllocationCallbacks>,
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
    pub inner: vk::ImageView,
    pub device: ash::Device,
    allocation_callbacks: Option<PAllocationCallbacks>,
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
    pub inner: ash::extensions::khr::Swapchain,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
}

impl_wrapper_deref!(SwapChainLoader, ash::extensions::khr::Swapchain);

pub struct SwapChain {
    pub inner: vk::SwapchainKHR,
    pub allocation_callbacks: Option<PAllocationCallbacks>,
    pub swapchain: ash::extensions::khr::Swapchain,
}
impl_wrapper_deref!(SwapChain, vk::SwapchainKHR);

impl Drop for SwapChain {
    fn drop(&mut self) {
        unsafe {
            let cb = get_cb!(self.allocation_callbacks);
            self.swapchain.destroy_swapchain(self.inner, cb)
        }
    }
}
