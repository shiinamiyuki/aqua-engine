use std::{cell::RefCell, collections::HashMap, sync::Arc};

use shaderc::{CompilationArtifact, Compiler, ShaderKind};

struct CompilationCache {
    pub compiler: Compiler,
    pub cache: HashMap<String, Arc<CompilationArtifact>>,
}

impl CompilationCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            compiler: Compiler::new().unwrap(),
        }
    }
    fn compile<'a>(
        &'a mut self,
        source: &String,
        shader_kind: ShaderKind,
        input_file_name: &str,
    ) -> Option<Arc<CompilationArtifact>> {
        if let Some(spirv) = self.cache.get(source) {
            return Some(spirv.clone());
        }
        let artifact =
            self.compiler
                .compile_into_spirv(&source, shader_kind, input_file_name, "main", None);

        match artifact {
            Ok(spirv) => {
                let spirv = Arc::new(spirv);
                self.cache.insert(source.clone(), spirv.clone());
                Some(spirv)
            }
            Err(err) => {
                eprintln!("Shader Compilation Failure: {}", err);
                None
            }
        }
    }
}
thread_local! {
    static CACHE: RefCell<CompilationCache> = RefCell::new(CompilationCache::new());
}

pub fn compile_shader_file(
    path: &std::path::Path,
    shader_kind: ShaderKind,
    device: &wgpu::Device,
) -> Option<wgpu::ShaderModule> {
    CACHE.with(|cache| {
        let source = std::fs::read_to_string(path).unwrap();
        let mut cache = cache.borrow_mut();
        let spirv = cache.compile(&source, shader_kind, path.to_str()?)?;
        let data = wgpu::util::make_spirv(spirv.as_binary_u8());
        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(path.to_str().unwrap()),
            source: data,
            flags: wgpu::ShaderFlags::default(),
        });
        Some(module)
    })
}
