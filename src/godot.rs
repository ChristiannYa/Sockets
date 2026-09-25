mod codec;
mod rel;

use godot::prelude::{ExtensionLibrary, gdextension};

struct BitpExension;

#[gdextension]
unsafe impl ExtensionLibrary for BitpExension {}
