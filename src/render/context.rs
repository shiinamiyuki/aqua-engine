use wgpu::SwapChainTexture;
use winit::window::Window;

pub struct DeviceContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}
pub struct RenderContext {
    pub device_ctx: DeviceContext,
    pub surface: wgpu::Surface,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl RenderContext {
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
                other => panic!("Unknown backend: {}", other),
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
                other => panic!("Unnknown power preference: {}", pref),
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
                        | wgpu::Features::PUSH_CONSTANTS
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits {
                        max_push_constant_size: 64,
                        ..wgpu::Limits::default()
                    },
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
        Self {
            device_ctx: DeviceContext { device, queue },
            surface,
            sc_desc,
            swap_chain,

            size,
        }
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self
            .device_ctx
            .device
            .create_swap_chain(&self.surface, &self.sc_desc);
    }
}

pub struct FrameContext {
    pub frame: SwapChainTexture,
}
