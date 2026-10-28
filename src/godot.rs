mod codec;
mod rel;
mod sim;
mod state;

use godot::prelude::{ExtensionLibrary, gdextension};

struct BitpExension;

#[gdextension]
unsafe impl ExtensionLibrary for BitpExension {}
