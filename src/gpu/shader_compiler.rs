use slang::{self, Downcast};


pub struct ShaderCompiler {
	global_session: slang::GlobalSession,
}

impl ShaderCompiler {
	pub fn new() -> Self {
		Self {
			global_session: slang::GlobalSession::new().unwrap(),
		}
	}

	pub fn compile(&self, file: &str, entry_point_name: &str) -> Vec<u8> {
		let search_path = std::ffi::CString::new("shaders").unwrap();

		let session_options = slang::OptionsBuilder::new()
			.optimization(slang::OptimizationLevel::High)
			.matrix_layout_row(true);

		let target_desc = slang::TargetDescBuilder::new()
			.format(slang::CompileTarget::Dxil)
			.profile(self.global_session.find_profile("sm_6_5"));

		let session_desc = slang::SessionDescBuilder::new()
			.targets(&[*target_desc])
			.search_paths(&[search_path.as_ptr()])
			.options(&session_options);

		let session = self.global_session.create_session(&session_desc).unwrap();

		let module = session.load_module(file).unwrap();
		let entry_point = module.find_entry_point_by_name(entry_point_name).unwrap();

		let program = session.create_composite_component_type(&[
			module.downcast().clone(), entry_point.downcast().clone(),
		]).unwrap();

		let linked_program = program.link().unwrap();

		linked_program.get_entry_point_code(0, 0).unwrap().as_slice().to_vec()
	}
}
