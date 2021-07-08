// use std::cell::RefCell;

// use wgpu::util::BufferInitDescriptor;
// use wgpu::util::DeviceExt;
// use lazy_static::lazy_static;
// struct GPUMatrix {
//     data: wgpu::Buffer,
//     shape: (u32, u32),
// }
// struct SGemmKernel {
//     bind_group_layout:wgpu::BindGroupLayout,
//     pipeline: wgpu::ComputePipeline,
// }

// // impl SGemmKernel {
// //     fn new()->Self{

// //     }
// // }
// struct MatrixKernels {
//     sgemm: SGemmKernel
// }
// // lazy_static! {
// //     static ref KERNELS: RefCell<Option<MatrixKernels>> = RefCell::new(None);
// // }
// impl GPUMatrix {
//     fn from_data(nrows: u32, ncols: u32, data: &[f32], device: &wgpu::Device) -> Self {
//         Self {
//             data: device.create_buffer_init(&BufferInitDescriptor {
//                 label: None,
//                 contents: bytemuck::cast_slice(data),
//                 usage: wgpu::BufferUsage::all(),
//             }),
//             shape: (nrows, ncols),
//         }
//     }

//     // *self = alpha * a * b + beta * *self
//     fn sgemm(&self, alpha: f32, a: &GPUMatrix, b: &GPUMatrix, beta: f32) {

//     }
// }
