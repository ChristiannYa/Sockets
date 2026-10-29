mod codec;
mod rel;
mod state;

use godot::prelude::{ExtensionLibrary, gdextension};

struct BitpExension;

#[gdextension]
unsafe impl ExtensionLibrary for BitpExension {}
