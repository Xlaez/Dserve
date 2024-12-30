mod enums;
mod implementations;
use enums::ProtocolState;
use ring::aead;
use std::{
    collections::{HashMap, VecDeque},
    net::UdpSocket,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
struct Packet {
    sequence: u32,
    ack: u32,
    ack_bits: u32,
    data: Vec<u8>,
    timestamp: Instant,
    attempts: u8,
}

pub struct PacketBuffer {
    incoming: VecDeque<Packet>,
    outgoing: VecDeque<Packet>,
    max_size: usize,
}

// Congestion control structure
pub struct CongestionControl {
    window_size: u32,
    threshold: u32,
    rtt: Duration,
    rtt_var: Duration,
    last_window_decrease: Instant,
}

struct EncryptionManager {
    key: aead::LessSafeKey,
    nonce_sequence: u64,
}

struct NetworkProtocol {
    socket: UdpSocket,
    state: ProtocolState,
    buffer: PacketBuffer,
    sequence_number: u32,
    ack_number: u32,
    congestion: CongestionControl,
    encryption: EncryptionManager,
    reliable_packets: HashMap<u32, Packet>,
    timeout: Duration,
}

fn main() -> std::io::Result<()> {
    let mut protocol = NetworkProtocol::new("127.0.0.1:3800")?;
    protocol.connect("127.0.0.1:3800")?;

    loop {
        protocol.update()?;
        // Using 60fps update rate
        std::thread::sleep(Duration::from_millis(16));
    }
}
