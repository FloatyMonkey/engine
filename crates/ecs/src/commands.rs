use super::{Entity, World};

#[derive(Default)]
pub struct Commands {
	commands: Vec<Box<dyn Command>>, // TODO: Optimize
}

impl Commands {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn push<C: Command + 'static>(&mut self, command: C) {
		self.commands.push(Box::new(command));
	}

	pub fn despawn(&mut self, entity: Entity) {
		self.push(Despawn { entity })
	}

	pub fn execute(&mut self, world: &mut World) {
		for mut command in self.commands.drain(..) {
			command.execute(world);
		}
	}
}

pub trait Command {
	fn execute(&mut self, world: &mut World);
}

pub struct Despawn {
	pub entity: Entity,
}

impl Command for Despawn {
	fn execute(&mut self, world: &mut World) {
		world.entity_mut(self.entity).despawn();
	}
}
