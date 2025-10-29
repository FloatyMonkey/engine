use std::collections::HashMap;

#[derive(Default)]
pub struct AssetServer {
	id: u64,
	assets: HashMap<u64, Box<dyn std::any::Any>>,
}

impl AssetServer {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn insert<T: Asset>(&mut self, asset: T) -> AssetId<T> {
		let handle = AssetId {
			id: self.id,
			phantom: std::marker::PhantomData,
		};
		self.assets.insert(handle.id, Box::new(asset));
		self.id += 1;
		handle
	}

	pub fn get<T: Asset>(&self, handle: &AssetId<T>) -> Option<&T> {
		self.assets.get(&handle.id).and_then(|asset| asset.downcast_ref::<T>())
	}

	pub fn get_mut<T: Asset>(&mut self, handle: &AssetId<T>) -> Option<&mut T> {
		self.assets.get_mut(&handle.id).and_then(|asset| asset.downcast_mut::<T>())
	}
}

pub trait Asset: std::any::Any {}

pub struct AssetId<T> {
	id: u64,
	phantom: std::marker::PhantomData<T>,
}

impl<T> AssetId<T> {
	pub fn id(&self) -> UntypedAssetId {
		self.id
	}
}

impl<T: Asset> Clone for AssetId<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Asset> Copy for AssetId<T> {}

pub type UntypedAssetId = u64;
