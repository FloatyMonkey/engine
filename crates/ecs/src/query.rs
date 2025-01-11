use crate::recursive;
use super::{Archetype, Component, Entity, World};
use std::{cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit, iter::FusedIterator};

/// Type that can be fetched from a [`World`] using a [`Query`].
pub trait QueryParam {
	/// The item type returned by this [`QueryParam`].
	type Item<'a>;

	/// Per archetype state used by this [`QueryParam`] to fetch [`QueryParam::Item`].
	type Fetch<'a>;

	fn matches_archetype(world: &World, archetype: &Archetype) -> bool;
	fn fetch<'a>(world: &'a World, archetype: &'a Archetype) -> Self::Fetch<'a>;
	fn item<'a>(fetch: &mut Self::Fetch<'a>, index: usize) -> Self::Item<'a>;
}

impl QueryParam for Entity {
	type Item<'a> = Entity;
	type Fetch<'a> = (&'a World, &'a Archetype);

	fn matches_archetype(_world: &World, _archetype: &Archetype) -> bool {
		true
	}

	fn fetch<'a>(world: &'a World, archetype: &'a Archetype) -> Self::Fetch<'a> {
		(world, archetype)
	}

	#[inline(always)]
	fn item<'a>(fetch: &mut Self::Fetch<'a>, index: usize) -> Self::Item<'a> {
		let (world, archetype) = fetch;
		let entity_index = archetype.entities[index];
		let entity_generation = world.entities[entity_index as usize].generation;
		Entity::new(entity_index, entity_generation)
	}
}

impl<T: Component> QueryParam for &T {
	type Item<'a> = &'a T;
	type Fetch<'a> = &'a [UnsafeCell<T>];

	fn matches_archetype(world: &World, archetype: &Archetype) -> bool {
		world.component_id::<T>().map_or(false, |id| archetype.contains(id))
	}

	fn fetch<'a>(world: &'a World, archetype: &'a Archetype) -> Self::Fetch<'a> {
		let component = archetype.component_index(world.component_id::<T>().unwrap()).unwrap();
		unsafe { archetype.get_slice(component) }
	}

	#[inline(always)]
	fn item<'a>(fetch: &mut Self::Fetch<'a>, index: usize) -> Self::Item<'a> {
		unsafe { &*fetch[index].get() }
	}
}

impl<T: Component> QueryParam for &mut T {
	type Item<'a> = &'a mut T;
	type Fetch<'a> = &'a [UnsafeCell<T>];

	fn matches_archetype(world: &World, archetype: &Archetype) -> bool {
		world.component_id::<T>().map_or(false, |id| archetype.contains(id))
	}

	fn fetch<'a>(world: &'a World, archetype: &'a Archetype) -> Self::Fetch<'a> {
		let component = archetype.component_index(world.component_id::<T>().unwrap()).unwrap();
		unsafe { archetype.get_slice(component) }
	}

	#[inline(always)]
	fn item<'a>(fetch: &mut Self::Fetch<'a>, index: usize) -> Self::Item<'a> {
		unsafe { &mut *fetch[index].get() }
	}
}

/// Returns a bool that indicates if the entity has the component `C`.
pub struct Has<C: Component>(PhantomData<C>);

impl<C: Component> QueryParam for Has<C> {
	type Item<'a> = bool;
	type Fetch<'a> = bool;

	fn matches_archetype(_world: &World, _archetype: &Archetype) -> bool {
		true
	}

	fn fetch<'a>(world: &'a World, archetype: &'a Archetype) -> Self::Fetch<'a> {
		world.component_id::<C>().map_or(false, |id| archetype.contains(id))
	}

	#[inline(always)]
	fn item<'a>(fetch: &mut Self::Fetch<'a>, _index: usize) -> Self::Item<'a> {
		*fetch
	}
}

impl<T: QueryParam> QueryParam for Option<T> {
	type Item<'a> = Option<T::Item<'a>>;
	type Fetch<'a> = Option<T::Fetch<'a>>;

	fn matches_archetype(_world: &World, _archetype: &Archetype) -> bool {
		true
	}

	fn fetch<'a>(world: &'a World, archetype: &'a Archetype) -> Self::Fetch<'a> {
		T::matches_archetype(world, archetype).then(|| T::fetch(world, archetype))
	}

	#[inline(always)]
	fn item<'a>(fetch: &mut Self::Fetch<'a>, index: usize) -> Self::Item<'a> {
		fetch.as_mut().map(|fetch| T::item(fetch, index))
	}
}

macro_rules! query_params_impl {
	($($name: ident),*) => {
		impl<$($name: QueryParam,)*> QueryParam for ($($name,)*) {
			type Item<'a> = ($($name::Item<'a>,)*);
			type Fetch<'a> = ($($name::Fetch<'a>,)*);

			fn matches_archetype(world: &World, archetype: &Archetype) -> bool {
				$($name::matches_archetype(world, archetype))&&*
			}

			fn fetch<'a>(world: &'a World, archetype: &'a Archetype) -> Self::Fetch<'a> {
				($($name::fetch(world, archetype),)*)
			}

			#[inline(always)]
			fn item<'a>(fetch: &mut Self::Fetch<'a>, index: usize) -> Self::Item<'a> {
				#[allow(non_snake_case)]
				let ($($name,)*) = fetch;
				($($name::item($name, index),)*)
			}
		}
	};
}

recursive! (query_params_impl, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

/// An [`Iterator`] over the items returned by a [`Query`].
pub struct QueryIter<'w, Q: QueryParam> {
	world: &'w World,
	archetypes: core::ops::Range<usize>,

	// State for the current archetype.
	fetch: MaybeUninit<Q::Fetch<'w>>,
	row: usize,
	len: usize,
}

impl<'w, Q: QueryParam> QueryIter<'w, Q> {
	fn new(world: &'w World) -> Self {
		Self {
			world,
			archetypes: 0..world.archetypes.len(),
			fetch: MaybeUninit::uninit(),
			row: 0,
			len: 0,
		}
	}
}

impl<'w, Q: QueryParam> Iterator for QueryIter<'w, Q> {
	type Item = Q::Item<'w>;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			// First iteration or reached the end of the current archetype.
			if self.row == self.len {
				let archetype_idx = self.archetypes.next()?;
				let archetype = &self.world.archetypes[archetype_idx];
				if archetype.is_empty() || !Q::matches_archetype(self.world, archetype) {
					continue;
				}
				self.fetch = MaybeUninit::new(Q::fetch(self.world, archetype));
				self.row = 0;
				self.len = archetype.len();
			}

			// SAFETY: `fetch` was initialized prior.
			let item = unsafe { Q::item(self.fetch.assume_init_mut(), self.row) };

			self.row += 1;
			return Some(item);
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.archetypes
			.clone()
			.map(|i| &self.world.archetypes[i])
			.filter(|archetype| !archetype.is_empty() && Q::matches_archetype(self.world, archetype))
			.map(|archetype| archetype.len())
			.sum::<usize>() + (self.len - self.row);

		(len, Some(len))
	}
}

impl<'w, Q: QueryParam> ExactSizeIterator for QueryIter<'w, Q> {}
impl<'w, Q: QueryParam> FusedIterator for QueryIter<'w, Q> {}

pub struct Query<'w, T: QueryParam> {
	world: &'w World,
	_phantom: PhantomData<T>,
}

impl<'w, T: QueryParam> Query<'w, T> {
	pub fn new(world: &'w World) -> Self {
		Self {
			world,
			_phantom: PhantomData,
		}
	}

	pub fn iter(&self) -> QueryIter<'w, T> {
		QueryIter::new(self.world)
	}
}

impl<'w, T: QueryParam> IntoIterator for &'w Query<'w, T> {
	type Item = T::Item<'w>;
	type IntoIter = QueryIter<'w, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
