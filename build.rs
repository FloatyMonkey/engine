fn main() {
	println!("cargo:rustc-link-arg=/EXPORT:D3D12SDKVersion,DATA");
	println!("cargo:rustc-link-arg=/EXPORT:D3D12SDKPath,DATA");
}
