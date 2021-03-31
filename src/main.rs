use glm::quat_euler_angles;
use na::indexing;

pub mod render;

use crate::render::mesh::{Mesh, MeshRenderer};
use crate::render::camera::Camera;
use crate::render::state::{State};
use tobj::load_obj;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

// use std::path::Path;
// use nalgebra_glm::{vec3, Vec3};
use futures::executor::block_on;
use nalgebra as na;
use nalgebra_glm as glm;
use std::time::Instant;
use std::{collections::HashMap, sync::Arc};
use wgpu::util::DeviceExt;

pub struct TriangleMesh {
    vertices: Vec<glm::Vec3>,
    normals: Vec<glm::Vec3>,
    texcoords: Vec<glm::IVec2>,
    indices: Vec<glm::UVec3>,
    normal_indices: Vec<glm::UVec3>,
    texcoord_indices: Vec<glm::UVec3>,
}

#[derive(Default, Copy, Clone)]
struct MVP {
    model: glm::Mat4,
    view: glm::Mat4,
    projection: glm::Mat4,
}

fn compute_normals(model: &mut TriangleMesh) {
    model.normals.clear();
    model.normal_indices.clear();
    let mut face_normals = vec![];
    let mut vertex_neighbors: HashMap<u32, Vec<u32>> = HashMap::new();
    for f in 0..model.indices.len() {
        let face = model.indices[f];
        // let face= glm::IVec3::from_rows(&[model.F.row(f)]);//[3 * f..3 * f + 3];
        for idx in face.iter() {
            if !vertex_neighbors.contains_key(idx) {
                vertex_neighbors.insert(*idx, vec![f as u32]);
            } else {
                vertex_neighbors.get_mut(idx).unwrap().push(f as u32);
            }
        }
        let triangle: Vec<glm::Vec3> = face
            .into_iter()
            .map(|idx| model.vertices[*idx as usize])
            .collect();
        let edge0: glm::Vec3 = triangle[1] - triangle[0];
        let edge1: glm::Vec3 = triangle[2] - triangle[0];
        let ng = glm::normalize(&glm::cross(&edge0, &edge1));
        face_normals.push(ng);
    }

    model.normals = (0..model.vertices.len())
        .into_iter()
        .map(|v| match vertex_neighbors.get(&(v as u32)) {
            None => glm::vec3(0.0, 0.0, 0.0),

            Some(faces) => {
                let ng_sum: glm::Vec3 = faces.into_iter().map(|f| face_normals[*f as usize]).sum();
                let ng = ng_sum / (faces.len() as f32);

                ng
            }
        })
        .collect();
}

fn load_model(obj_file: &str) -> Vec<TriangleMesh> {
    let (models, materials) = tobj::load_obj(&obj_file, true).expect("Failed to load file");

    let mut imported_models = vec![];
    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;

        println!("model[{}].name = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!(
            "Size of model[{}].num_face_indices: {}",
            i,
            mesh.num_face_indices.len()
        );
        let mut vertices = vec![];
        let mut normals = vec![];
        // let mut indices = vec![];
        assert!(mesh.positions.len() % 3 == 0);
        for v in 0..mesh.positions.len() / 3 {
            vertices.push(glm::vec3(
                mesh.positions[3 * v],
                mesh.positions[3 * v + 1],
                mesh.positions[3 * v + 2],
            ));
        }
        let mut indices = vec![];
        for f in 0..mesh.indices.len() / 3 {
            indices.push(glm::UVec3::new(
                mesh.indices[3 * f],
                mesh.indices[3 * f + 1],
                mesh.indices[3 * f + 2],
            ));
        }
        if !mesh.normals.is_empty() {
            for i in 0..mesh.normals.len() / 3 {
                normals.push(glm::vec3(
                    mesh.normals[3 * i],
                    mesh.normals[3 * i + 1],
                    mesh.normals[3 * i + 2],
                ));
            }
        }
        let mut imported = TriangleMesh {
            vertices: vertices,
            normals: normals,
            indices: indices,
            normal_indices: vec![],
            texcoords: vec![],
            texcoord_indices: vec![],
        };
        if mesh.normals.is_empty() {
            compute_normals(&mut imported);
        }

        // let mut next_face = 0;
        // for f in 0..mesh.num_face_indices.len() {
        //     assert!(mesh.num_face_indices[f] == 3);
        //     let end = next_face + mesh.num_face_indices[f] as usize;
        //     let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();
        //     println!("    face[{}] = {:?}", f, face_indices);
        //     next_face = end;
        // }
        imported_models.push(imported);
    }

    imported_models
}


fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title(String::from("Arukas Engine"))
        .build(&event_loop)
        .unwrap();
    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window));
    let args: Vec<String> = std::env::args().collect();
    let models = if args.len() > 1 {
        load_model(&args[1])
    } else {
        load_model("./living_room.obj")
    };
    let renderers: Vec<MeshRenderer> = models
        .into_iter()
        .map(|model| MeshRenderer::new(&mut state, &Mesh::from_triangle_mesh(&model)))
        .collect();
    let camera = Camera {
        eye: glm::vec3(0.0, 0.6, 3.0),
        center: glm::vec3(0.0, 0.6, 2.0),
        aspect: 16.0 / 9.0,
        fovy: glm::pi::<f32>() / 2.0,
        up: glm::vec3(0.0, 1.0, 0.0),
        znear: 0.1,
        zfar: 100.0,
    };
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            state.update();
            let render_result = state.render(&camera, renderers.iter());
            match render_result {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
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
            if !state.input(event) {
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
