use std::marker::PhantomData;

use wgpu::util::DeviceExt;

use super::DeviceContext;
pub trait BufferData: Default + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    type Native;
    fn new(value: &Self::Native) -> Self;
}

pub fn create_uniform_bind_group_layout(
    ctx: &DeviceContext,
    binding: u32,
    visibility: wgpu::ShaderStage,
    label: Option<&str>,
) -> wgpu::BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label,
        })
}

pub fn create_storage_bind_group_layout(
    ctx: &DeviceContext,
    binding: u32,
    visibility: wgpu::ShaderStage,
    read_only: bool,
    label: Option<&str>,
) -> wgpu::BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label,
        })
}

pub struct Buffer<T>
where
    T: BufferData,
{
    pub buffer: wgpu::Buffer,
    // pub bind_group: wgpu::BindGroup,
    // pub binding: u32,
    // pub layout: wgpu::BindGroupLayout,
    marker: PhantomData<T>,
}
impl<T> Buffer<T>
where
    T: BufferData,
{
    pub fn bindgroup_layout_entry(
        &self,
        binding: u32,
        visibility: wgpu::ShaderStage,
        ty: wgpu::BindingType,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        }
    }
    pub fn bindgroup_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: binding,
            resource: self.buffer.as_entire_binding(),
        }
    }
    pub fn new_uniform_buffer(
        ctx: &DeviceContext,
        init: &[T],
        label: Option<&str>,
    ) -> Self {
        let device = &ctx.device;
        // let layout = create_uniform_bind_group_layout(ctx, binding, visibility, label);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label.map(|s: &str| {
                s.to_owned().push_str(".uniform");
                s
            }),
            contents: bytemuck::cast_slice(init),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: binding,
        //         resource: buffer.as_entire_binding(),
        //     }],
        //     label: label.map(|s: &str| {
        //         s.to_owned().push_str(".uniform_bind_group");
        //         s
        //     }),
        // });
        Self {
            buffer,
            // layout,
            // binding,
            marker: PhantomData,
        }
    }
    pub fn new_storage_buffer(
        ctx: &DeviceContext,
        init: &[T],
        label: Option<&str>,
    ) -> Self {
        let device = &ctx.device;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label.map(|s: &str| {
                s.to_owned().push_str(".buffer");
                s
            }),
            contents: bytemuck::cast_slice(init),
            usage: wgpu::BufferUsage::STORAGE
                | wgpu::BufferUsage::COPY_DST
                | wgpu::BufferUsage::COPY_SRC,
        });
        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: binding,
        //         resource: buffer.as_entire_binding(),
        //     }],
        //     label: label.map(|s: &str| {
        //         s.to_owned().push_str(".buffer_bind_group");
        //         s
        //     }),
        // });
        Self {
            buffer,
            // bind_group,
            // layout,
            // binding,
            marker: PhantomData,
        }
    }

    pub fn upload(&self, ctx: &DeviceContext, values: &[T]) {
        ctx.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(values));
    }
}
