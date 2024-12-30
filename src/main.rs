mod implementations;
use std::time::{Duration, Instant};

use ring::aead;

#[derive(Debug, Clone)]
struct Packet {
    sequence: u32,
    ack: u32,
    ack_bits: u32,
    data: Vec<u8>,
    timestamp: Instant,
    attempts: u8,
}

// Congestion control structure
struct CongestionControl {
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
