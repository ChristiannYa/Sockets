mod codec;
mod dedup;
mod pending;

use godot::prelude::{ExtensionLibrary, gdextension};

struct BitpExension;

#[gdextension]
unsafe impl ExtensionLibrary for BitpExension {}
