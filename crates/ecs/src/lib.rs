mod commands;
mod query;
mod world;

pub use commands::*;
pub use query::*;
pub use world::*;

/// Recursive macro treating arguments as a progression.
///
/// Expansion of recursive!(macro, A, B, C) is equivalent to the expansion of sequence:
/// - macro!(A)
/// - macro!(A, B)
/// - macro!(A, B, C)
#[macro_export]
macro_rules! recursive {
	($macro: ident, $args: ident) => {
		$macro!{$args}
	};
	($macro: ident, $first: ident, $($rest: ident),*) => {
		$macro!{$first, $($rest),*}
		recursive!{$macro, $($rest),*}
	};
}
