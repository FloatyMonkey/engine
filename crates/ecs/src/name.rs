pub struct Name {
	pub name: String,
}

impl Name {
	pub fn new(name: impl ToString) -> Self {
		Self {
			name: name.to_string(),
		}
	}
}
