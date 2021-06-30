use std::{cell::RefCell, collections::HashMap, sync::Arc};

use shaderc::{
    CompilationArtifact, CompileOptions, Compiler, IncludeCallbackResult, ResolvedInclude,
    ShaderKind,
};

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
        program_name: &String,
        source: &String,
        shader_kind: ShaderKind,
        input_file_name: &str,
        options: Option<&CompileOptions>,
    ) -> Option<Arc<CompilationArtifact>> {
        if let Some(spirv) = self.cache.get(program_name) {
            return Some(spirv.clone());
        }
        let mut options = if let Some(opt) = options {
            opt.clone().unwrap()
        } else {
            CompileOptions::new().unwrap()
        };
        options.set_include_callback(
            |name, _include_type, _src, _depth| -> IncludeCallbackResult {
                let filename = format!("src/shaders/{}", name);
                let content = std::fs::read_to_string(&filename).unwrap_or_else(|_| {
                    panic!("failed to resolved include {}", name);
                });
                Ok(ResolvedInclude {
                    content,
                    resolved_name: filename,
                })
            },
        );
        let artifact = self.compiler.compile_into_spirv(
            &source,
            shader_kind,
            input_file_name,
            "main",
            Some(&options),
        );

        match artifact {
            Ok(spirv) => {
                let spirv = Arc::new(spirv);
                self.cache.insert(program_name.clone(), spirv.clone());
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

pub fn compile_shader_file<S : Into<String>>(
    path: &std::path::Path,
    program_name: S,
    shader_kind: ShaderKind,
    device: &wgpu::Device,
    options: Option<&CompileOptions>,
) -> Option<wgpu::ShaderModule> {
    CACHE.with(|cache| {
        let source = std::fs::read_to_string(path).unwrap();
        let mut cache = cache.borrow_mut();
        let spirv = cache.compile(&program_name.into(), &source, shader_kind, path.to_str()?, options)?;
        let data = wgpu::util::make_spirv(spirv.as_binary_u8());
        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(path.to_str().unwrap()),
            source: data,
            flags: wgpu::ShaderFlags::default(),
        });
        Some(module)
    })
}
