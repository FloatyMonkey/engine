use crate::recursive;
use super::{Query, QueryParam};

use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::num::NonZeroU32;

pub type EntityId = u32;
pub type ComponentId = Entity;

/// Type that can store data for an [`Entity`].
pub trait Component: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Component for T {}

trait ComponentVec: Send + Sync {
	fn to_any(&self) -> &dyn Any;
	fn to_any_mut(&mut self) -> &mut dyn Any;
	fn swap_remove(&mut self, index: EntityId);
	fn migrate(&mut self, entity_index: EntityId, other_archetype: &mut dyn ComponentVec);
	fn new_same_type(&self) -> Box<dyn ComponentVec>;
	fn push_ptr(&mut self, ptr: *const u8);
}

impl<C: Component> ComponentVec for Vec<C> {
	fn to_any(&self) -> &dyn Any {
		self
	}

	fn to_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn swap_remove(&mut self, index: EntityId) {
		self.swap_remove(index as usize);
	}

	fn migrate(&mut self, entity_index: EntityId, other_component_vec: &mut dyn ComponentVec) {
		let data = self.swap_remove(entity_index as usize);
		component_vec_to_mut(other_component_vec).push(data);
	}

	fn new_same_type(&self) -> Box<dyn ComponentVec> {
		Box::new(Vec::<C>::new())
	}

	fn push_ptr(&mut self, ptr: *const u8) {
		unsafe {
			self.push(ptr.cast::<C>().read());
		}
	}
}

fn component_vec_to_mut<C: 'static>(c: &mut dyn ComponentVec) -> &mut Vec<C> {
	c.to_any_mut()
		.downcast_mut::<Vec<C>>()
		.unwrap()
}

pub struct ComponentStore {
	pub id: ComponentId,
	data: Box<dyn ComponentVec>,
}

impl ComponentStore {
	pub fn new<C: Component>(id: ComponentId) -> Self {
		Self {
			id,
			data: Box::new(Vec::<C>::new()),
		}
	}

	pub fn new_same_type(&self) -> Self {
		Self {
			id: self.id,
			data: self.data.new_same_type(),
		}
	}
}

type ArchetypeId = usize;

pub struct Archetype {
	pub entities: Vec<EntityId>,
	pub components: Vec<ComponentStore>,
}

impl Archetype {
	pub fn new() -> Self {
		Self {
			entities: Vec::new(),
			components: Vec::new(),
		}
	}

	fn swap_remove(&mut self, index: EntityId) -> EntityId {
		for c in self.components.iter_mut() {
			c.data.swap_remove(index)
		}

		let moved = *self.entities.last().unwrap();
		self.entities.swap_remove(index as usize);
		moved
	}

	pub fn component_index(&self, id: ComponentId) -> Option<usize> {
		self.components.iter().position(|c| c.id == id)
	}

	pub unsafe fn get_slice<C: Component>(&self, component: usize) -> &[UnsafeCell<C>] {
		let data = self.components[component].data
			.to_any()
			.downcast_ref::<Vec<C>>()
			.unwrap();

		unsafe { std::mem::transmute(data.as_slice()) }
	}

	pub fn contains(&self, component_id: ComponentId) -> bool {
		self.components.iter().any(|c| c.id == component_id)
	}
	
	pub fn len(&self) -> usize {
		self.entities.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entities.is_empty()
	}
}

#[derive(Clone, Copy)]
pub struct EntityLocation {
	archetype_id: ArchetypeId,
	archetype_row: EntityId,
}

impl EntityLocation {
	const EMPTY: Self = Self {
		archetype_id: 0,
		archetype_row: 0,
	};
}

#[derive(Clone, Copy)]
pub struct EntityInfo {
	pub generation: NonZeroU32,
	pub location: EntityLocation,
}

impl EntityInfo {
	const EMPTY: Self = Self {
		generation: NonZeroU32::MIN,
		location: EntityLocation::EMPTY,
	};
}

/// A handle to an entity within a [`World`].
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Entity {
	index: EntityId,
	generation: NonZeroU32,
}

impl Entity {
	pub const fn new(index: EntityId, generation: NonZeroU32) -> Self {
		Self { index, generation }
	}

	pub const fn index(&self) -> EntityId {
		self.index
	}

	pub const fn generation(&self) -> EntityId {
		self.generation.get()
	}
}

impl std::fmt::Debug for Entity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Entity({}, {})", self.index(), self.generation())
	}
}

impl std::fmt::Display for Entity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Entity({}, {})", self.index(), self.generation())
	}
}

/// Adds two numbers, wrapping to 1 instead of 0 on overflow.
const fn wrapping_add_nonzero(lhs: NonZeroU32, rhs: u32) -> NonZeroU32 {
	let (sum, carry) = lhs.get().overflowing_add(rhs);
	let ret = sum + carry as u32;
	// SAFETY: Adding the carry flag will offset overflows to start at 1 instead of 0.
	unsafe { NonZeroU32::new_unchecked(ret) }
}

pub struct World {
	pub archetypes: Vec<Archetype>,
	pub entities: Vec<EntityInfo>,
	free_entities: Vec<EntityId>,
	components: HashMap<TypeId, ComponentId>,
	dyn_components: HashMap<ComponentId, ComponentStore>, // TODO: ComponentStore used here is always empty, we only is it for new_same_type.
	bundle_id_to_archetype: HashMap<u64, ArchetypeId>,
}

impl World {
	pub fn new() -> Self {
		Self {
			archetypes: Vec::new(),
			entities: Vec::new(),
			free_entities: Vec::new(),
			components: HashMap::new(),
			dyn_components: HashMap::new(),
			bundle_id_to_archetype: HashMap::new(),
		}
	}

	fn init_component<C: Component>(&mut self) -> ComponentId {
		if let Some(id) = self.components.get(&TypeId::of::<C>()) {
			*id
		} else {
			let id = self.alloc_entity();
			// TODO: We don't set the entities location, maybe not a big deal?
			self.components.insert(TypeId::of::<C>(), id);
			self.dyn_components.insert(id, ComponentStore::new::<C>(id));
			id
		}
	}

	pub fn component_id<C: Component>(&self) -> Option<ComponentId> {
		self.components.get(&TypeId::of::<C>()).copied()
	}

	fn alloc_entity(&mut self) -> Entity {
		if let Some(index) = self.free_entities.pop() {
			Entity { index, generation: self.entities[index as usize].generation }
		} else {
			self.entities.push(EntityInfo::EMPTY);
			debug_assert!(self.entities.len() <= EntityId::MAX as usize);
			Entity { index: (self.entities.len() - 1) as EntityId, generation: NonZeroU32::MIN }
		}
	}

	fn free_entity(&mut self, entity: Entity) -> Option<EntityLocation> {
		let entity_info = &mut self.entities[entity.index as usize];

		if entity.generation != entity_info.generation {
			return None;
		}

		entity_info.generation = wrapping_add_nonzero(entity_info.generation, 1);
		self.free_entities.push(entity.index);

		Some(std::mem::replace(&mut entity_info.location, EntityLocation::EMPTY))
	}

	fn entity_location(&self, entity: Entity) -> Option<EntityLocation> {
		let entity_info = self.entities.get(entity.index as usize)?;

		if entity.generation != entity_info.generation {
			return None;
		}

		Some(entity_info.location)
	}

	pub fn spawn(&mut self, bundle: impl Bundle) -> EntityMut {
		let entity = self.alloc_entity();
		let location = spawn_in_world(self, bundle, entity.index);
		self.entities[entity.index as usize].location = location;

		EntityMut { world: self, location, entity }
	}

	pub fn query<Q: QueryParam>(&self) -> Query<Q> {
		Query::new(self)
	}

	/// Returns an [`EntityRef`] that exposes read-only operations for the given `entity`.
	/// Panics if the `entity` does not exist. Use [`World::get_entity_mut`] to check for existence instead of panic-ing.
	#[inline]
	#[track_caller]
	pub fn entity(&self, entity: Entity) -> EntityRef {
		match self.get_entity(entity) {
			Some(entity) => entity,
			None => panic!("{entity:?} does not exist"),
		}
	}

	/// Returns an [`EntityRef`] that exposes read-only operations for the given `entity`.
	/// Returns `None` if the `entity` does not exist. Use [`World::entity_mut`] to panic instead of unwrapping.
	pub fn get_entity(&self, entity: Entity) -> Option<EntityRef> {
		let location = self.entity_location(entity)?;
		// SAFETY: `entity` exists and `location` is its location.
		Some(unsafe { EntityRef::new(self, location, entity) })
	}

	/// Returns an [`EntityMut`] that exposes read and write operations for the given `entity`.
	/// Panics if the `entity` does not exist. Use [`World::get_entity_mut`] to check for existence instead of panic-ing.
	#[inline]
	#[track_caller]
	pub fn entity_mut(&mut self, entity: Entity) -> EntityMut {
		match self.get_entity_mut(entity) {
			Some(entity) => entity,
			None => panic!("{entity:?} does not exist"),
		}
	}

	/// Returns an [`EntityMut`] that exposes read and write operations for the given `entity`.
	/// Returns `None` if the `entity` does not exist. Use [`World::entity_mut`] to panic instead of unwrapping.
	pub fn get_entity_mut(&mut self, entity: Entity) -> Option<EntityMut> {
		let location = self.entity_location(entity)?;
		// SAFETY: `entity` exists and `location` is its location.
		Some(unsafe { EntityMut::new(self, location, entity) })
	}
}

/// # Singletons
/// They are created by adding the component to its own entity id.
impl World {
	pub fn add_singleton<C: Component>(&mut self, component: C) {
		let component_id = self.init_component::<C>();
		let location = spawn_in_world(self, (component,), component_id.index);
		self.entities[component_id.index as usize].location = location;
	}

	pub fn get_singleton<C: Component>(&self) -> Option<&C> {
		let id = self.component_id::<C>()?;
		self.get_entity(id)?.get::<C>()
	}

	pub fn get_singleton_mut<C: Component>(&mut self) -> Option<&mut C> {
		let id = self.component_id::<C>()?;
		self.get_entity_mut(id)?.get_mut::<C>()
	}
}

/// A read-only reference to an entity.
pub struct EntityRef<'w> {
	world: &'w World,
	location: EntityLocation,
	entity: Entity,
}

impl<'w> EntityRef<'w> {
	unsafe fn new(world: &'w World, location: EntityLocation, entity: Entity) -> Self {
		Self { world, location, entity }
	}

	/// Returns the id of this entity.
	#[must_use]
	pub fn id(self) -> Entity {
		self.entity
	}

	/// Returns the [`Archetype`] of this entity.
	pub fn archetype(&self) -> &Archetype {
		&self.world.archetypes[self.location.archetype_id]
	}

	/// Returns `true` if this entity has a component of type `C`. Otherwise returns `false`.
	pub fn contains<C: Component>(&self) -> bool {
		self.world.component_id::<C>().map_or(false, |id| self.archetype().contains(id))
	}

	/// Gets access to the component of type `C` on this entity.
	/// Returns `None` if the entity does not have a component of type `C`.
	pub fn get<C: Component>(&self) -> Option<&'w C> {
		let archetype = self.archetype();
		let component_index = archetype.component_index(self.world.component_id::<C>()?)?;

		unsafe {
			let data = archetype.get_slice::<C>(component_index);
			Some(&*data[self.location.archetype_row as usize].get())
		}
	}
}

/// A mutable reference to an entity.
pub struct EntityMut<'w> {
	world: &'w mut World,
	location: EntityLocation,
	entity: Entity,
}

impl<'w> EntityMut<'w> {
	unsafe fn new(world: &'w mut World, location: EntityLocation, entity: Entity) -> Self {
		Self { world, location, entity }
	}

	/// Returns the id of this entity.
	#[must_use]
	pub fn id(self) -> Entity {
		self.entity
	}

	/// Returns the [`Archetype`] of this entity.
	pub fn archetype(&self) -> &Archetype {
		&self.world.archetypes[self.location.archetype_id]
	}

	/// Returns `true` if this entity has a component of type `C`. Otherwise returns `false`.
	pub fn contains<C: Component>(&self) -> bool {
		self.world.component_id::<C>().map_or(false, |id| self.archetype().contains(id))
	}

	/// Gets access to the component of type `C` on this entity.
	/// Returns `None` if the entity does not have a component of type `C`.
	pub fn get<C: Component>(&'w self) -> Option<&'w C> {
		self.as_ref().get()
	}

	/// Gets mutable access to the component of type `C` on this entity.
	/// Returns `None` if the entity does not have a component of type `C`.
	pub fn get_mut<C: Component>(&mut self) -> Option<&'w mut C> {
		let archetype = self.archetype();
		let component_index = archetype.component_index(self.world.component_id::<C>()?)?;

		unsafe {
			let data = archetype.get_slice::<C>(component_index);
			Some(&mut *data[self.location.archetype_row as usize].get())
		}
	}

	pub fn despawn(self) {
		if let Some(location) = self.world.free_entity(self.entity) {
			let moved_entity = self.world.archetypes[location.archetype_id]
				.swap_remove(location.archetype_row);
			self.world.entities[moved_entity as usize].location = location;
		}
	}

	fn as_ref(&'w self) -> EntityRef<'w> {
		unsafe { EntityRef::new(self.world, self.location, self.entity) }
	}
}

fn calculate_bundle_id(types: &[ComponentId]) -> u64 {
	let mut s = DefaultHasher::new();
	types.hash(&mut s);
	s.finish()
}

pub trait DynamicBundle {
	fn component_ids(&self, world: &mut World, ids: &mut impl FnMut(ComponentId));
	fn write(self, func: &mut impl FnMut(*mut u8));
}

pub trait Bundle: DynamicBundle + Send + Sync + 'static {}

macro_rules! bundle_impl {
	($($name: ident),*) => {
		impl< $($name: Component),*> Bundle for ($($name,)*) {}

		impl<$($name: Component),*> DynamicBundle for ($($name,)*) {
			fn component_ids(&self, world: &mut World, ids: &mut impl FnMut(ComponentId)) {
				$(ids(world.init_component::<$name>());)*
			}

			fn write(self, func: &mut impl FnMut(*mut u8)) {
				#[allow(non_snake_case)]
				let ($(mut $name,)*) = self;
				$(
					func((&mut $name as *mut $name).cast::<u8>(),);
					std::mem::forget($name);
				)*
			}
		}
	}
}

recursive! (bundle_impl, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

fn spawn_in_world<B: DynamicBundle>(world: &mut World, bundle: B, entity_index: EntityId) -> EntityLocation {
	let mut components = Vec::new();
	bundle.component_ids(world, &mut |id| components.push(id));
	let unsorted_components = components.clone();
	components.sort_unstable();

	let bundle_id = calculate_bundle_id(&components);

	let archetype_index = if let Some(archetype) = world.bundle_id_to_archetype.get(&bundle_id) {
		*archetype
	} else {
		let mut archetype = Archetype::new();

		for component_id in components.iter() {
			archetype.components.push(world.dyn_components.get(component_id).unwrap().new_same_type());
		}

		let index = world.archetypes.len();

		world.bundle_id_to_archetype.insert(bundle_id, index);
		world.archetypes.push(archetype);
		index
	};
	
	let archetype = &mut world.archetypes[archetype_index];
	archetype.entities.push(entity_index);
	let mut component_i = 0;
	bundle.write(&mut |ptr| {
		let component_id = unsorted_components[component_i];
		let component_index = archetype.components.iter().position(|c| c.id == component_id).unwrap();
		archetype.components[component_index].data.push_ptr(ptr);
		component_i += 1;
	});

	EntityLocation {
		archetype_id: archetype_index,
		archetype_row: (world.archetypes[archetype_index].len() - 1) as EntityId
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn entity_niche_optimization() {
		assert_eq!(
			size_of::<Entity>(),
			size_of::<Option<Entity>>()
		);
	}
}
