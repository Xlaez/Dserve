use std::{
    collections::HashMap,
    io::Write,
    net::UdpSocket,
    time::{Duration, Instant},
};

use flate2::{
    write::{ZlibDecoder, ZlibEncoder},
    Compression,
};

use crate::{
    definitions::{CongestionControl, EncryptionManager, NetworkProtocol, Packet, PacketBuffer},
    enums::ProtocolState,
};

impl NetworkProtocol {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            state: ProtocolState::Idle,
            buffer: PacketBuffer::new(1024),
            sequence_number: 0,
            ack_number: 0,
            congestion: CongestionControl::new(),
            encryption: EncryptionManager::new(),
            reliable_packets: HashMap::new(),
            timeout: Duration::from_secs(5),
        })
    }

    pub fn connect(&mut self, remote_addr: &str) -> std::io::Result<()> {
        self.socket.connect(remote_addr)?;
        self.state = ProtocolState::Connecting;

        let connect_packet = Packet {
            sequence: self.sequence_number,
            data: vec![0x01],
            timestamp: Instant::now(),
            ack: 0,
            ack_bits: 0,
            attempts: 0,
        };

        self.buffer.push_outgoing(connect_packet);
        self.sequence_number += 1;
        Ok(())
    }

    pub fn update_state(&mut self) {
        match self.state {
            ProtocolState::Connecting => {
                if let Some(packet) = self.buffer.incoming.front() {
                    if packet.data[0] == 0x02 {
                        self.state = ProtocolState::Connected;
                    }
                }

                if let Some(packet) = self.buffer.outgoing.front() {
                    if packet.timestamp.elapsed() > self.timeout {
                        self.state = ProtocolState::Idle;
                    }
                }
            }
            ProtocolState::Connected => {
                // Todo: Handle normal protocol operation
            }
            _ => {}
        }
    }

    pub fn send_reliable(&mut self, data: Vec<u8>) -> std::io::Result<()> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data)?;
        let compressed = encoder.finish()?;

        let encrypted = self.encryption.encrypt(&compressed);

        let packet = Packet {
            sequence: self.sequence_number,
            ack: self.ack_number,
            ack_bits: self.generate_ack_bits(),
            data: encrypted,
            timestamp: Instant::now(),
            attempts: 0,
        };

        self.reliable_packets
            .insert(self.sequence_number, packet.clone());
        self.buffer.push_outgoing(packet);
        self.sequence_number += 1;

        Ok(())
    }

    pub fn generate_ack_bits(&self) -> u32 {
        let mut ack_bits = 0u32;

        for i in 1..=32 {
            let seq = self.ack_number.wrapping_sub(i);
            if self.reliable_packets.contains_key(&seq) {
                ack_bits |= 1 << (i - 1);
            }
        }

        ack_bits
    }

    pub fn handle_ack(&mut self, ack: u32, ack_bits: u32) {
        self.reliable_packets.remove(&ack);
        self.congestion.on_ack();

        for i in 1..=32 {
            if (ack_bits & (1 << (i - 1))) != 0 {
                let seq = ack.wrapping_sub(i);
                self.reliable_packets.remove(&seq);
            }
        }
    }

    pub fn update(&mut self) -> std::io::Result<()> {
        let now = Instant::now();
        let mut retransmit = Vec::new();

        for (seq, packet) in self.reliable_packets.iter_mut() {
            if now.duration_since(packet.timestamp) > self.congestion.rtt * 2 {
                if packet.attempts < 5 {
                    packet.attempts += 1;
                    packet.timestamp = now;
                    retransmit.push(packet.clone());
                    self.congestion.on_loss();
                } else {
                    self.state = ProtocolState::Disconnecting;
                }
            }
        }

        for packet in retransmit {
            self.buffer.push_outgoing(packet);
        }

        loop {
            let mut buf = [0u8; 2048];
            match self.socket.recv_from(&mut buf) {
                Ok((size, _)) => {
                    let mut encrypted_data = buf[..size].to_vec();

                    if let Ok(decrypted) = self.encryption.decrypt(&mut encrypted_data) {
                        let mut decoder = ZlibDecoder::new(Vec::new());
                        decoder.write_all(&decrypted)?;

                        let decompressed = decoder.finish()?;

                        let packet = Packet {
                            sequence: self.sequence_number,
                            ack: self.ack_number,
                            ack_bits: 0,
                            data: decompressed,
                            timestamp: Instant::now(),
                            attempts: 0,
                        };

                        self.buffer.push_incoming(packet.clone());
                        self.handle_ack(packet.ack, packet.ack_bits);
                    }
                }

                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }

        let mut sent = 0;
        while let Some(packet) = self.buffer.outgoing.pop_front() {
            if sent >= self.congestion.window_size {
                break;
            }

            self.socket.send(&packet.data)?;
            sent += 1;
        }

        self.update_state();

        Ok(())
    }
}
