use shaderc::{Compiler, ShaderKind};


pub fn compile_shader_file(
    path: &std::path::Path,
    shader_kind: ShaderKind,
    device: &wgpu::Device,
    compiler: &mut Compiler,
) -> Option<wgpu::ShaderModule> {
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
}
