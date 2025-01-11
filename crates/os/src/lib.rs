pub mod win32;

#[cfg(target_os = "windows")]
pub use win32 as platform;

#[derive(Clone, Copy)]
pub struct Rect<T> {
	pub x: T,
	pub y: T,
	pub width: T,
	pub height: T,
}

#[derive(Clone, Copy)]
pub struct Point<T> {
	pub x: T,
	pub y: T,
}

impl<T> From<Point<T>> for [T; 2] {
	fn from(p: Point<T>) -> Self {
		[p.x, p.y]
	}
}

pub type Size<T> = Point<T>;

#[derive(Clone, Copy)]
pub enum MouseButton {
	Left,
	Middle,
	Right,
}

#[derive(Clone, Copy)]
pub enum Key {
	A, B, C, D, E, F, G, H, I, J, K, L, M,
	N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

	ArrowLeft,
	ArrowRight,
	ArrowUp,
	ArrowDown,

	Escape,
	Tab,
	Backspace,
	Enter,
	Space,

	Insert,
	Delete,
	Home,
	End,
	PageUp,
	PageDown,

	Minus,
	Plus,

	Num0, Num1, Num2, Num3, Num4,
	Num5, Num6, Num7, Num8, Num9,

	F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
	F11, F12, F13, F14, F15, F16, F17, F18, F19, F20,
}

#[derive(Clone, Copy)]
pub enum Event {
	Key { key: Key, pressed: bool },
	Text { character: char },
	MouseButton { button: MouseButton, pressed: bool },
	MouseWheel { delta: [f32; 2] },
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Cursor {
	None,
	Arrow,
	Crosshair,
	Hand,
	Help,
	Text,
	Wait,
	ResizeAll,
	ResizeEw,
	ResizeNs,
	ResizeNeSw,
	ResizeNwSe,
	NotAllowed,
}

#[derive(Clone)]
pub struct MonitorInfo {
	pub name: String,
	pub rect: Rect<i32>,
	pub scale_factor: f32,
	pub primary: bool,
}

#[derive(Clone)]
pub struct WindowDesc {
	pub title: String,
	pub rect: Rect<i32>,
}

pub struct NativeHandle(pub u64);

pub trait App {
	type Window: Window;

	/// Creates a new app instance.
	fn new() -> Self;

	/// Creates a new window.
	fn create_window(&mut self, desc: &WindowDesc) -> Self::Window;

	/// Should be called every frame to update app and windows state.
	/// Returns false when the app is requested to close.
	fn run(&mut self) -> bool;

	/// Returns the events that have occured since the last call to [`App::run`].
	fn events(&self) -> Vec<Event>;

	/// Returns the mouse position relative to the top-left corner of the desktop.
	fn mouse_pos(&self) -> Point<i32>;

	/// Returns info about all monitors connected to the system.
	fn enumerate_monitors() -> Vec<MonitorInfo>;

	/// Sets the mouse cursor icon.
	fn set_cursor(&mut self, cursor: &Cursor);
}

pub trait Window {
	fn title(&self) -> String;
	fn set_title(&self, title: &str);

	/// Returns the position of the top-left corner of the window relative to the top-left corner of the desktop.
	fn position(&self) -> Point<i32>;

	/// Sets the position of the top-left corner of the window relative to the top-left corner of the desktop.
	fn set_position(&self, pos: Point<i32>);

	/// Returns the size of the window's client area.
	fn size(&self) -> Size<u32>;

	/// Sets the size of the window's client area.
	fn set_size(&self, size: Size<i32>);

	fn is_minimized(&self) -> bool;
	fn minimize(&self);

	fn is_maximized(&self) -> bool;
	fn maximize(&self);

	/// Returns true if the window has keyboard focus.
	fn is_focused(&self) -> bool;

	/// Brings the window to the front and sets keyboard focus.
	fn focus(&self);

	/// Returns the mouse position relative to the top-left corner of the window's client area.
	fn mouse_pos_client(&self, mouse_pos: Point<i32>) -> Point<i32>;

	/// Returns the dpi scale factor for the monitor the window is currently on.
	fn scale_factor(&self) -> f32;

	/// Returns the platform native handle for the window.
	fn native_handle(&self) -> NativeHandle;
}
