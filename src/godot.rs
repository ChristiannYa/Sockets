mod codec;
mod intent;
mod rel;

use godot::prelude::{ExtensionLibrary, gdextension};

struct BitpExension;

#[gdextension]
unsafe impl ExtensionLibrary for BitpExension {}
