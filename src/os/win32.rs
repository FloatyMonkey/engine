use windows::{
	core::*,
	Win32::Foundation::*,
	Win32::Graphics::Gdi::{
		ClientToScreen, EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, ScreenToClient, ValidateRect,
		HDC, HMONITOR, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST
	},
	Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
	Win32::System::LibraryLoader::*, Win32::UI::Controls::*, Win32::UI::HiDpi::*,
	Win32::UI::Input::KeyboardAndMouse::*, Win32::UI::WindowsAndMessaging::*,
	Win32::System::Com::CoInitialize,
};

use std::ffi::{OsStr, OsString};
use std::os::windows::prelude::{OsStrExt, OsStringExt};

#[derive(Clone)]
pub struct App {
	window_class_wide: Vec<u16>,
	hinstance: HINSTANCE,
	mouse_pos: super::Point<i32>,
	cursor: super::Cursor,
	proc_data: ProcData,
}

#[derive(Clone)]
pub struct Window {
	hwnd: HWND,
	ws: WINDOW_STYLE,
	wsex: WINDOW_EX_STYLE,
}

#[derive(Clone)]
struct ProcData {
	mouse_hwnd: HWND,
	mouse_tracked: bool,
	mouse_down: [bool; 3],
	events: Vec<super::Event>,
}

impl Drop for Window {
	fn drop(&mut self) {
		unsafe {
			// TODO: Unwrapping this crashes
			// Presumably because `App` gets dropped before `Window`
			let _ = DestroyWindow(self.hwnd);
		}
	}
}

impl Drop for App {
	fn drop(&mut self) {
		unsafe {
			UnregisterClassW(PCWSTR(self.window_class_wide.as_ptr()), self.hinstance).unwrap();
		}
	}
}

fn adjust_window_rect(rect: &super::Rect<i32>, ws: WINDOW_STYLE, wsex: WINDOW_EX_STYLE) -> super::Rect<i32> {
	let mut rc = RECT {
		left: rect.x,
		top: rect.y,
		right: rect.x + rect.width,
		bottom: rect.y + rect.height,
	};
	unsafe {
		AdjustWindowRectEx(&mut rc, ws, false, wsex).unwrap();
	}
	super::Rect::<i32> {
		x: rc.left,
		y: rc.top,
		width: rc.right - rc.left,
		height: rc.bottom - rc.top,
	}
}

pub fn encode_wide(string: impl AsRef<OsStr>) -> Vec<u16> {
	string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

pub fn decode_wide(mut wide_c_string: &[u16]) -> OsString {
	if let Some(null_pos) = wide_c_string.iter().position(|c| *c == 0) {
		wide_c_string = &wide_c_string[..null_pos];
	}

	OsString::from_wide(wide_c_string)
}

impl App {
	fn set_capture(&mut self, window: HWND) {
		unsafe {
			let any_down = self.proc_data.mouse_down.iter().any(|v| *v);
			if !any_down && GetCapture() == HWND(0) {
				SetCapture(window);
			}
		}
	}

	fn release_capture(&mut self, window: HWND) {
		unsafe {
			let any_down = self.proc_data.mouse_down.iter().any(|v| *v);
			if !any_down && GetCapture() == window {
				ReleaseCapture().unwrap();
			}
		}
	}
}

impl super::App for App {
	type Window = Window;

	fn new() -> Self {
		unsafe {
			CoInitialize(None).unwrap();

			let window_class = "Window Class".to_string();
			let window_class_wide = encode_wide(&window_class);

			let instance = GetModuleHandleW(None).unwrap();

			SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
			SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE).unwrap();

			let wc = WNDCLASSW {
				hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
				hInstance: instance.into(),
				lpszClassName: PCWSTR(window_class_wide.as_ptr()),
				style: CS_HREDRAW | CS_VREDRAW,
				lpfnWndProc: Some(wndproc),
				..Default::default()
			};

			if RegisterClassW(&wc) == 0 {
				panic!();
			}

			App {
				window_class_wide,
				hinstance: instance.into(),
				mouse_pos: super::Point{ x: 0, y: 0 },
				cursor: super::Cursor::Arrow,
				proc_data: ProcData {
					mouse_hwnd: HWND(0),
					mouse_tracked: false,
					mouse_down: [false; 3],
					events: Vec::new(),
				},
			}
		}
	}

	fn create_window(&mut self, desc: &super::WindowDesc) -> Window {
		unsafe {
			let ws = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
			let wsex = WINDOW_EX_STYLE::default();
			let rect = adjust_window_rect(&desc.rect, ws, wsex);

			let window_name = encode_wide(&desc.title);

			let hwnd = CreateWindowExW(
				wsex,
				PCWSTR(self.window_class_wide.as_ptr()),
				PCWSTR(window_name.as_ptr()),
				ws,
				rect.x,
				rect.y,
				rect.width,
				rect.height,
				HWND::default(),
				None,
				self.hinstance,
				Some(self as *const _ as _), // TODO: Ptr might break
			);

			let enable_dark_mode = BOOL::from(true);
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &enable_dark_mode as *const BOOL as *const _, std::mem::size_of::<BOOL>() as _).unwrap();

			Window {
				hwnd,
				ws,
				wsex,
			}
		}
	}

	fn run(&mut self) -> bool {
		self.proc_data.events.clear();

		let mut mouse_pos = POINT::default();
		unsafe { GetCursorPos(&mut mouse_pos).unwrap(); }
		self.mouse_pos = super::Point {
			x: mouse_pos.x,
			y: mouse_pos.y,
		};

		let mut msg = MSG::default();

		unsafe {
			while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
				TranslateMessage(&msg);
				DispatchMessageW(&msg);

				if msg.message == WM_QUIT {
					return false;
				}
			}
		}

		true
	}

	fn events(&self) -> Vec<super::Event> {
		self.proc_data.events.clone()
	}

	fn mouse_pos(&self) -> super::Point<i32> {
		self.mouse_pos
	}

	fn enumerate_monitors() -> Vec<super::MonitorInfo> {
		let mut monitors: Vec<super::MonitorInfo> = Vec::new();
		unsafe {
			EnumDisplayMonitors(HDC(0), None, Some(monitor_enum_proc), LPARAM(&mut monitors as *mut _ as _));
		}
		monitors
	}

	fn set_cursor(&mut self, cursor: &super::Cursor) {
		if self.cursor == *cursor {
			return;
		}
		self.cursor = *cursor;
		unsafe {
			if let Ok(cursor) = map_cursor(cursor).map_or(Ok(HCURSOR(0)), |c| LoadCursorW(HINSTANCE::default(), c)) {
				SetCursor(cursor);
			}
		}
	}
}

impl super::Window for Window {
	fn title(&self) -> String {
		let length = unsafe { GetWindowTextLengthW(self.hwnd) } + 1;
		let mut buffer = vec![0; length as usize];
		unsafe { GetWindowTextW(self.hwnd, &mut buffer); }
		decode_wide(&buffer).to_string_lossy().to_string()
	}

	fn set_title(&self, title: &str) {
		let title = encode_wide(title);
		unsafe { SetWindowTextW(self.hwnd, PCWSTR(title.as_ptr())).unwrap(); }
	}

	fn position(&self) -> super::Point<i32> {
		let mut pos = POINT { x: 0, y: 0 };
		unsafe { ClientToScreen(self.hwnd, &mut pos); }
		super::Point { x: pos.x, y: pos.y }
	}

	fn set_position(&self, pos: super::Point<i32>) {
		let mut rect = RECT {
			left: pos.x,
			top: pos.y,
			right: pos.x,
			bottom: pos.y,
		};
		let flags = SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE;
		unsafe {
			AdjustWindowRectEx(&mut rect, self.ws, false, self.wsex).unwrap();
			SetWindowPos(self.hwnd, HWND(0), rect.left, rect.top, 0, 0, flags).unwrap();
		}
	}

	fn size(&self) -> super::Size<u32> {
		let mut rect = RECT::default();
		unsafe { GetClientRect(self.hwnd, &mut rect).unwrap(); }
		super::Size {
			x: (rect.right - rect.left) as u32,
			y: (rect.bottom - rect.top) as u32,
		}
	}

	fn set_size(&self, size: super::Point<i32>) {
		let mut rect = RECT {
			left: 0,
			top: 0,
			right: size.x,
			bottom: size.y,
		};
		let flags = SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE;
		unsafe {
			AdjustWindowRectEx(&mut rect, self.ws, false, self.wsex).unwrap();
			SetWindowPos(self.hwnd, HWND(0), 0, 0, rect.right - rect.left, rect.bottom - rect.top, flags).unwrap();
		}
	}

	fn is_minimized(&self) -> bool {
		unsafe { IsIconic(self.hwnd) != false }
	}

	fn minimize(&self) {
		unsafe { ShowWindow(self.hwnd, SW_MINIMIZE); }
	}

	fn is_maximized(&self) -> bool {
		unsafe { IsZoomed(self.hwnd) != false }
	}

	fn maximize(&self) {
		unsafe { ShowWindow(self.hwnd, SW_MAXIMIZE); }
	}

	fn is_focused(&self) -> bool {
		unsafe { GetForegroundWindow() == self.hwnd }
	}

	fn focus(&self) {
		unsafe {
			SetActiveWindow(self.hwnd);
			BringWindowToTop(self.hwnd).unwrap();
			SetForegroundWindow(self.hwnd);
			SetFocus(self.hwnd);
		}
	}

	fn mouse_pos_client(&self, mouse_pos: super::Point<i32>) -> super::Point<i32> {
		let mut mp = POINT {
			x: mouse_pos.x,
			y: mouse_pos.y,
		};
		unsafe { ScreenToClient(self.hwnd, &mut mp); }
		super::Point { x: mp.x, y: mp.y }
	}

	fn scale_factor(&self) -> f32 {
		let monitor = unsafe { MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST) };
		
		let mut x_dpi: u32 = 0;
		let mut y_dpi: u32 = 0;
		if unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x_dpi, &mut y_dpi) }.is_err() {
			return 1.0;
		}

		(x_dpi as f32) / 96.0
	}

	fn native_handle(&self) -> super::NativeHandle {
		super::NativeHandle(self.hwnd.0 as _)
	}
}

unsafe extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
	let app = &mut *(GetWindowLongPtrW(window, GWLP_USERDATA) as *mut App);
	let proc_data = &mut app.proc_data;

	match message as u32 {
		WM_CREATE => {
			let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
			SetWindowLongPtrW(window, GWLP_USERDATA, create_struct.lpCreateParams as _);
			LRESULT(0)
		}
		WM_DESTROY => {
			PostQuitMessage(0);
			LRESULT(0)
		}
		WM_MOUSEMOVE => {
			proc_data.mouse_hwnd = window;
			if !proc_data.mouse_tracked {
				// Call TrackMouseEvent to receive WM_MOUSELEAVE events
				TrackMouseEvent(&mut TRACKMOUSEEVENT {
					cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
					dwFlags: TME_LEAVE,
					hwndTrack: window,
					dwHoverTime: HOVER_DEFAULT,
				}).unwrap();
				proc_data.mouse_tracked = true;
			}
			LRESULT(0)
		}
		WM_MOUSELEAVE => {
			proc_data.mouse_hwnd = HWND(0);
			proc_data.mouse_tracked = false;
			LRESULT(0)
		}
		WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
			proc_data.mouse_down[0] = true;
			proc_data.events.push(super::Event::MouseButton { button: super::MouseButton::Left, pressed: true });
			app.set_capture(window);
			LRESULT(0)
		}
		WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
			proc_data.mouse_down[1] = true;
			proc_data.events.push(super::Event::MouseButton { button: super::MouseButton::Right, pressed: true });
			app.set_capture(window);
			LRESULT(0)
		}
		WM_MBUTTONDOWN | WM_MBUTTONDBLCLK => {
			proc_data.mouse_down[2] = true;
			proc_data.events.push(super::Event::MouseButton { button: super::MouseButton::Middle, pressed: true });
			app.set_capture(window);
			LRESULT(0)
		}
		WM_LBUTTONUP => {
			proc_data.mouse_down[0] = false;
			proc_data.events.push(super::Event::MouseButton{ button: super::MouseButton::Left, pressed: false });
			app.release_capture(window);
			LRESULT(0)
		}
		WM_RBUTTONUP => {
			proc_data.mouse_down[1] = false;
			proc_data.events.push(super::Event::MouseButton{ button: super::MouseButton::Right, pressed: false });
			app.release_capture(window);
			LRESULT(0)
		}
		WM_MBUTTONUP => {
			proc_data.mouse_down[2] = false;
			proc_data.events.push(super::Event::MouseButton{ button: super::MouseButton::Middle, pressed: false });
			app.release_capture(window);
			LRESULT(0)
		}
		WM_MOUSEWHEEL => {
			let value = (wparam.0 >> 16) as i16;
			let value = value as f32 / WHEEL_DELTA as f32;
			proc_data.events.push(super::Event::MouseWheel { delta: [0.0, value] });
			LRESULT(0)
		}
		WM_MOUSEHWHEEL => {
			let value = (wparam.0 >> 16) as i16;
			let value = value as f32 / WHEEL_DELTA as f32;
			proc_data.events.push(super::Event::MouseWheel { delta: [value, 0.0] });
			LRESULT(0)
		}
		WM_PAINT => {
			ValidateRect(window, None);
			LRESULT(0)
		}
		WM_CHAR => {
			if wparam.0 > 0 && wparam.0 < 0x10000 {
				proc_data.events.push(super::Event::Text { character: char::from_u32(wparam.0 as u32).unwrap() });
			}
			LRESULT(0)
		}
		WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
			let down = (message == WM_KEYDOWN) || (message == WM_SYSKEYDOWN);

			let vk = VIRTUAL_KEY(wparam.0 as u16);

			if let Some(key) = map_key(vk) {
				proc_data.events.push(super::Event::Key { key, pressed: down });
			}

			LRESULT(0)
		}
		WM_SETCURSOR => {
			if (lparam.0 & 0xffff) as u32 == HTCLIENT {
				if let Ok(cursor) = map_cursor(&app.cursor).map_or(Ok(HCURSOR(0)), |c| LoadCursorW(HINSTANCE::default(), c)) {
					SetCursor(cursor);
				}
				LRESULT(1)
			} else {
				DefWindowProcW(window, message, wparam, lparam)
			}
		}
		_ => DefWindowProcW(window, message, wparam, lparam),
	}
}

unsafe extern "system" fn monitor_enum_proc(monitor: HMONITOR, _hdc: HDC, _lprect: *mut RECT, lparam: LPARAM) -> BOOL {
	let monitors = &mut *(lparam.0 as *mut Vec<super::MonitorInfo>);

	let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
	info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

	if GetMonitorInfoW(monitor, &mut info as *mut _ as *mut _) == false {
		return false.into();
	}

	let mut x_dpi: u32 = 0;
	let mut y_dpi: u32 = 0;
	let dpi_scale = if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x_dpi, &mut y_dpi).is_ok() {
		(x_dpi as f32) / 96.0
	} else {
		1.0
	};

	let rc_monitor = info.monitorInfo.rcMonitor;

	monitors.push(super::MonitorInfo {
		name: decode_wide(&info.szDevice).to_string_lossy().to_string(),
		rect: super::Rect {
			x: rc_monitor.left,
			y: rc_monitor.top,
			width: rc_monitor.right - rc_monitor.left,
			height: rc_monitor.bottom - rc_monitor.top,
		},
		scale_factor: dpi_scale,
		primary: (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
	});

	true.into()
}

fn map_cursor(cursor: &super::Cursor) -> Option<PCWSTR> {
	Some(match cursor {
		super::Cursor::None       => return None,
		super::Cursor::Arrow      => IDC_ARROW,
		super::Cursor::Crosshair  => IDC_CROSS,
		super::Cursor::Hand       => IDC_HAND,
		super::Cursor::Help       => IDC_HELP,
		super::Cursor::Text       => IDC_IBEAM,
		super::Cursor::Wait       => IDC_WAIT,
		super::Cursor::ResizeAll  => IDC_SIZEALL,
		super::Cursor::ResizeEw   => IDC_SIZEWE,
		super::Cursor::ResizeNs   => IDC_SIZENS,
		super::Cursor::ResizeNeSw => IDC_SIZENESW,
		super::Cursor::ResizeNwSe => IDC_SIZENWSE,
		super::Cursor::NotAllowed => IDC_NO,
	})
}

fn map_key(key: VIRTUAL_KEY) -> Option<super::Key> {
	use super::Key;

	Some(match key {
		VK_A => Key::A,
		VK_B => Key::B,
		VK_C => Key::C,
		VK_D => Key::D,
		VK_E => Key::E,
		VK_F => Key::F,
		VK_G => Key::G,
		VK_H => Key::H,
		VK_I => Key::I,
		VK_J => Key::J,
		VK_K => Key::K,
		VK_L => Key::L,
		VK_M => Key::M,
		VK_N => Key::N,
		VK_O => Key::O,
		VK_P => Key::P,
		VK_Q => Key::Q,
		VK_R => Key::R,
		VK_S => Key::S,
		VK_T => Key::T,
		VK_U => Key::U,
		VK_V => Key::V,
		VK_W => Key::W,
		VK_X => Key::X,
		VK_Y => Key::Y,
		VK_Z => Key::Z,

		VK_LEFT => Key::ArrowLeft,
		VK_RIGHT => Key::ArrowRight,
		VK_UP => Key::ArrowUp,
		VK_DOWN => Key::ArrowDown,

		VK_ESCAPE => Key::Escape,
		VK_TAB => Key::Tab,
		VK_BACK => Key::Backspace,
		VK_RETURN => Key::Enter,
		VK_SPACE => Key::Space,

		VK_INSERT => Key::Insert,
		VK_DELETE => Key::Delete,
		VK_HOME => Key::Home,
		VK_END => Key::End,
		VK_PRIOR => Key::PageUp,
		VK_NEXT => Key::PageDown,

		VK_OEM_MINUS => Key::Minus,
		VK_OEM_PLUS => Key::Plus,

		VK_0 | VK_NUMPAD0 => Key::Num0,
		VK_1 | VK_NUMPAD1 => Key::Num1,
		VK_2 | VK_NUMPAD2 => Key::Num2,
		VK_3 | VK_NUMPAD3 => Key::Num3,
		VK_4 | VK_NUMPAD4 => Key::Num4,
		VK_5 | VK_NUMPAD5 => Key::Num5,
		VK_6 | VK_NUMPAD6 => Key::Num6,
		VK_7 | VK_NUMPAD7 => Key::Num7,
		VK_8 | VK_NUMPAD8 => Key::Num8,
		VK_9 | VK_NUMPAD9 => Key::Num9,

		VK_F1 => Key::F1,
		VK_F2 => Key::F2,
		VK_F3 => Key::F3,
		VK_F4 => Key::F4,
		VK_F5 => Key::F5,
		VK_F6 => Key::F6,
		VK_F7 => Key::F7,
		VK_F8 => Key::F8,
		VK_F9 => Key::F9,
		VK_F10 => Key::F10,
		VK_F11 => Key::F11,
		VK_F12 => Key::F12,
		VK_F13 => Key::F13,
		VK_F14 => Key::F14,
		VK_F15 => Key::F15,
		VK_F16 => Key::F16,
		VK_F17 => Key::F17,
		VK_F18 => Key::F18,
		VK_F19 => Key::F19,
		VK_F20 => Key::F20,

		_ => return None,
	})
}
