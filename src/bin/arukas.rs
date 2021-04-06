use arukas::{geometry::load_model, glm, render::{Camera, ColorAttachment, LookAtCamera, Mesh, OribitalCamera, Perspective, RenderContext, RenderInput, RenderPass, SimpleRenderPass, Size, fovx_to_fovy}};

use arukas::render::GPUMesh;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

// use std::path::Path;
// use nalgebra_glm::{vec3, Vec3};
use futures::executor::block_on;
use std::time::Instant;
use std::{collections::HashMap, sync::Arc};
use wgpu::util::DeviceExt;

struct App {
    ctx: RenderContext,
    size: winit::dpi::PhysicalSize<u32>,
    gpu_meshes: Vec<GPUMesh>,
    camera: OribitalCamera,
    render_pass: SimpleRenderPass,
    last_cursor_pos: Option<winit::dpi::PhysicalPosition<f64>>,
    is_key_down: bool,
}
impl App {
    fn new(window: &Window) -> App {
        let mut ctx = block_on(RenderContext::new(&window));
        let models = if std::env::args().count() > 1 {
            let args: Vec<String> = std::env::args().collect();
            load_model(&args[1])
        } else {
            load_model("./living_room.obj")
        };
        let gpu_meshes: Vec<GPUMesh> = models
            .into_iter()
            .map(|model| GPUMesh::new(&mut ctx, &Mesh::from_triangle_mesh(&model)))
            .collect();
        // let camera = LookAtCamera {
        //     eye: glm::vec3(0.0, 0.6, 3.0),
        //     center: glm::vec3(0.0, 0.6, 2.0),
        //     up: glm::vec3(0.0, 1.0, 0.0),
        //     perspective: Perspective {
        //         aspect: 16.0 / 9.0,
        //         fovy: glm::pi::<f32>() / 2.0,

        //         znear: 0.1,
        //         zfar: 100.0,
        //     },
        // };
        let fovx = 120.0f32.to_radians();
        let perspective = Perspective {
            aspect: 16.0 / 9.0,
            fovy: fovx_to_fovy(fovx, 16.0 / 9.0),

            znear: 0.1,
            zfar: 100.0,
        };
        let camera = OribitalCamera {
            perspective,
            center: glm::vec3(0.0, 0.6, 2.0),
            radius: 1.0,
            phi: 0.0,
            theta: glm::pi::<f32>() * 0.5,
        };
        let render_pass = SimpleRenderPass::new(&ctx);
        App {
            ctx,
            size: window.inner_size(),
            gpu_meshes,
            camera,
            render_pass,
            last_cursor_pos: None,
            is_key_down: false,
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                device_id,
                state: ElementState::Released,
                button: MouseButton::Left,
                modifiers,
            } => {
                self.is_key_down = false;
                true
            }
            WindowEvent::MouseInput {
                device_id,
                state: ElementState::Pressed,
                button: MouseButton::Left,
                modifiers,
            } => {
                self.is_key_down = true;
                true
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
                modifiers: _,
            } => {
                if self.last_cursor_pos.is_none() {
                    self.last_cursor_pos = Some(*position);
                }
                if let Some(last_pos) = self.last_cursor_pos {
                    let diff = glm::vec2(position.x - last_pos.x, position.y - last_pos.y);
                    let dphi = diff.x / 2000.0 * glm::pi::<f64>();
                    let dtheta = diff.y / 1000.0 * glm::pi::<f64>();
                    self.camera.theta += dtheta as f32;
                    self.camera.phi += dphi as f32;
                }
                if self.is_key_down {
                    self.last_cursor_pos = Some(*position);
                } else {
                    self.last_cursor_pos = None;
                }

                true
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                is_synthetic,
            } => false,
            _ => false,
        }
    }
    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = size;
        self.ctx.resize(self.size)
    }
    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.ctx.swap_chain.get_current_frame()?.output;
        let input = RenderInput {
            attachements: vec![ColorAttachment { view: &frame.view }],
            meshes: &self.gpu_meshes[..],
        };
        self.render_pass.render(
            Size(self.size.width, self.size.height),
            &mut self.ctx,
            &self.camera,
            &input,
        );

        Ok(())
    }
    fn update(&mut self) {}
}
fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(String::from("Arukas Engine"))
        .with_resizable(false)
        .with_inner_size(winit::dpi::PhysicalSize::<u32>::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    // Since main can't be async, we're going to need to block
    let mut app = App::new(&window);

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            app.update();
            let render_result = app.render();
            // let render_result = state.render(&camera, renderers.iter());
            match render_result {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SwapChainError::Lost) => app.resize(app.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !app.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        app.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        app.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
