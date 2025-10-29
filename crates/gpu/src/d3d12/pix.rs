use std::ffi::CString;
use windows::{Win32::Graphics::Direct3D12::*, Win32::System::LibraryLoader::*, core::*};

type ProcAddress = unsafe extern "system" fn() -> isize;

type BeginEventOnCommandList = unsafe extern "system" fn(*const std::ffi::c_void, u64, PSTR) -> i32;
type EndEventOnCommandList = unsafe extern "system" fn(*const std::ffi::c_void) -> i32;
type SetMarkerOnCommandList = unsafe extern "system" fn(*const std::ffi::c_void, u64, PSTR) -> i32;

#[derive(Clone, Copy)]
pub struct WinPixEventRuntime {
	begin_event: BeginEventOnCommandList,
	end_event: EndEventOnCommandList,
	set_marker: SetMarkerOnCommandList,
}

impl WinPixEventRuntime {
	pub fn load() -> Option<Self> {
		unsafe {
			let module = LoadLibraryA(s!("WinPixEventRuntime.dll")).ok()?;

			Some(Self {
				begin_event: std::mem::transmute::<ProcAddress, BeginEventOnCommandList>(
					GetProcAddress(module, s!("PIXBeginEventOnCommandList"))?,
				),
				end_event: std::mem::transmute::<ProcAddress, EndEventOnCommandList>(
					GetProcAddress(module, s!("PIXEndEventOnCommandList"))?,
				),
				set_marker: std::mem::transmute::<ProcAddress, SetMarkerOnCommandList>(
					GetProcAddress(module, s!("PIXSetMarkerOnCommandList"))?,
				),
			})
		}
	}

	pub fn begin_event_on_command_list(
		&self,
		command_list: &ID3D12GraphicsCommandList7,
		color: u64,
		name: &str,
	) {
		let name = CString::new(name).unwrap();
		unsafe { (self.begin_event)(command_list.as_raw(), color, PSTR(name.as_ptr() as _)) };
	}

	pub fn end_event_on_command_list(&self, command_list: &ID3D12GraphicsCommandList7) {
		unsafe { (self.end_event)(command_list.as_raw()) };
	}

	pub fn set_marker_on_command_list(
		&self,
		command_list: &ID3D12GraphicsCommandList7,
		color: u64,
		name: &str,
	) {
		let name = CString::new(name).unwrap();
		unsafe { (self.set_marker)(command_list.as_raw(), color, PSTR(name.as_ptr() as _)) };
	}
}
