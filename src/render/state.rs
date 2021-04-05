use crate::render::camera::{Camera, UniformViewProjection};
use crate::render::mesh::{MeshRenderer, Vertex};
use crate::render::texture::Texture;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};
pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub depth_texture: Texture, // vertex_buffer: wgpu::Buffer,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let backend = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
            match backend.to_lowercase().as_str() {
                "vulkan" => wgpu::BackendBit::VULKAN,
                "metal" => wgpu::BackendBit::METAL,
                "dx12" => wgpu::BackendBit::DX12,
                "dx11" => wgpu::BackendBit::DX11,
                "gl" => wgpu::BackendBit::GL,
                "webgpu" => wgpu::BackendBit::BROWSER_WEBGPU,
                other => panic!("Unknown backend: {}", other)
            }
        } else {
            wgpu::BackendBit::PRIMARY
        };
        let instance = wgpu::Instance::new(backend);
        let surface = unsafe { instance.create_surface(window) };
        let power_pref = if let Ok(pref) = std::env::var("WGPU_POWER_PREF") {
            match pref.to_lowercase().as_str() {
                "low" => wgpu::PowerPreference::LowPower,
                "high" => wgpu::PowerPreference::HighPerformance,
                other => panic!("Unnknown power preference: {}", pref)
            }
        } else {
            wgpu::PowerPreference::default()
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty()
                        // | wgpu::Features::UNSIZED_BINDING_ARRAY
                        | wgpu::Features::SAMPLED_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
                        | wgpu::Features::SAMPLED_TEXTURE_ARRAY_DYNAMIC_INDEXING
                        | wgpu::Features::PUSH_CONSTANTS,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_src = std::fs::read_to_string("src/shader.vert").unwrap();
        let fs_src = std::fs::read_to_string("src/shader.frag").unwrap();
        let mut compiler = shaderc::Compiler::new().unwrap();
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
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });
        let uniforms = UniformViewProjection::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });
        let depth_texture = Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
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
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            uniform_buffer, // vertex_buffer,
            uniform_bind_group,
            depth_texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            Texture::create_depth_texture_from_sc(&self.device, &self.sc_desc, "depth_texture");
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
        // todo!()
    }

    pub fn render<'a, I>(
        &mut self,
        camera: &Camera,
        mesh_renderers: I,
    ) -> Result<(), wgpu::SwapChainError>
    where
        I: std::iter::Iterator<Item = &'a MeshRenderer>,
    {
        let mut uniforms = UniformViewProjection::new();
        uniforms.update_from_camera(&camera);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            // 1.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // NEW!
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline); // 2.
                                                             // render_pass.draw(0..3, 0..1); // 3.
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            
            for m in mesh_renderers {
                m.render(&mut render_pass);
            }
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
