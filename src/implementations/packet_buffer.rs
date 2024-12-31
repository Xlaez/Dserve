use std::collections::VecDeque;

use crate::definitions::{Packet, PacketBuffer};

impl PacketBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            incoming: VecDeque::new(),
            outgoing: VecDeque::new(),
            max_size,
        }
    }

    pub fn push_incoming(&mut self, packet: Packet) -> bool {
        if self.incoming.len() < self.max_size {
            self.incoming.push_back(packet);
            true
        } else {
            false
        }
    }

    pub fn push_outgoing(&mut self, packet: Packet) -> bool {
        if self.outgoing.len() < self.max_size {
            self.outgoing.push_back(packet);
            true
        } else {
            false
        }
    }
}
