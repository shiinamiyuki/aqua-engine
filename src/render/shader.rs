use std::cell::RefCell;

use shaderc::{Compiler, ShaderKind};

thread_local! {
    pub static COMPILER: RefCell<Compiler> = RefCell::new(shaderc::Compiler::new().unwrap());
}

pub fn compile_shader_file(
    path: &std::path::Path,
    shader_kind: ShaderKind,
    device: &wgpu::Device,
) -> Option<wgpu::ShaderModule> {
    COMPILER.with(|compiler| {
        let mut compiler = compiler.borrow_mut();
        let src = std::fs::read_to_string(path).ok()?;
        let artifact = compiler.compile_into_spirv(&src, shader_kind, path.to_str()?, "main", None);
        match artifact {
            Ok(spriv) => {
                let data = wgpu::util::make_spirv(spriv.as_binary_u8());
                let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some(path.to_str().unwrap()),
                    source: data,
                    flags: wgpu::ShaderFlags::default(),
                });
                Some(module)
            }
            Err(err) => {
                eprintln!("Shader Compilation Failure: {}", err);
                None
            }
        }
    })
}
