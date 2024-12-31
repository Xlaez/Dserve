use enums::ProtocolState;
use ring::aead;
use std::{
    collections::{HashMap, VecDeque},
    net::UdpSocket,
    time::{Duration, Instant},
};

use crate::enums;

#[derive(Debug, Clone)]
pub struct Packet {
    pub sequence: u32,
    pub ack: u32,
    pub ack_bits: u32,
    pub data: Vec<u8>,
    pub timestamp: Instant,
    pub attempts: u8,
}

pub struct PacketBuffer {
    pub incoming: VecDeque<Packet>,
    pub outgoing: VecDeque<Packet>,
    pub max_size: usize,
}

// Congestion control structure
pub struct CongestionControl {
    pub window_size: u32,
    pub threshold: u32,
    pub rtt: Duration,
    pub rtt_var: Duration,
    pub last_window_decrease: Instant,
}

pub struct EncryptionManager {
    pub key: aead::LessSafeKey,
    pub nonce_sequence: u64,
}

pub struct NetworkProtocol {
    pub socket: UdpSocket,
    pub state: ProtocolState,
    pub buffer: PacketBuffer,
    pub sequence_number: u32,
    pub ack_number: u32,
    pub congestion: CongestionControl,
    pub encryption: EncryptionManager,
    pub reliable_packets: HashMap<u32, Packet>,
    pub timeout: Duration,
}
