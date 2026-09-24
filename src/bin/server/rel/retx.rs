use std::{collections::HashMap, net::SocketAddr};

use bitp::rel::retx::PendingPacket;

pub type PendingPackets = HashMap<(u8, SocketAddr), PendingPacket>;
