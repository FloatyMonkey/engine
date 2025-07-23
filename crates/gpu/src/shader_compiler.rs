use shader_slang::{self as slang, Downcast};

pub struct ShaderCompiler {
	global_session: slang::GlobalSession,
	backend: super::Backend,
}

impl ShaderCompiler {
	pub fn new(backend: super::Backend) -> Self {
		Self {
			global_session: slang::GlobalSession::new().unwrap(),
			backend,
		}
	}

	pub fn compile(&self, file: &str, entry_point_name: &str) -> Vec<u8> {
		let search_path = std::ffi::CString::new("shaders").unwrap();

		let session_options = slang::CompilerOptions::default()
			.optimization(slang::OptimizationLevel::High)
			.matrix_layout_row(true);

		let target_desc = slang::TargetDesc::default()
			.format(match self.backend {
				super::Backend::D3D12 => slang::CompileTarget::Dxil,
				super::Backend::Vulkan => slang::CompileTarget::Spirv,
			})
			.profile(self.global_session.find_profile(match self.backend {
				super::Backend::D3D12 => "sm_6_5",
				super::Backend::Vulkan => "glsl_450",
			}));

		let targets = [target_desc];
		let search_paths = [search_path.as_ptr()];

		let session_desc = slang::SessionDesc::default()
			.targets(&targets)
			.search_paths(&search_paths)
			.options(&session_options);

		let session = self.global_session.create_session(&session_desc).unwrap();

		let module = session.load_module(file).unwrap();
		let entry_point = module.find_entry_point_by_name(entry_point_name).unwrap();

		let program = session.create_composite_component_type(&[
			module.downcast().clone(), entry_point.downcast().clone(),
		]).unwrap();

		let linked_program = program.link().unwrap();

		let code = linked_program.entry_point_code(0, 0).unwrap().as_slice().to_vec();

		code
	}
}
