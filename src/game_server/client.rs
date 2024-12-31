use std::{collections::VecDeque, time::Duration};

use crate::definitions::NetworkProtocol;

use super::types::{GameMessage, GameState, PlayerState, Vector2};

pub struct GameClient {
    pub protocol: NetworkProtocol,
    pub state: Option<GameState>,
    pub player_id: Option<u32>,
    pub interpolation_buffer: VecDeque<GameState>,
}

impl GameClient {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        Ok(Self {
            protocol: NetworkProtocol::new(addr)?,
            state: None,
            player_id: None,
            interpolation_buffer: VecDeque::with_capacity(128),
        })
    }

    pub fn connect(&mut self, server_addr: &str) -> std::io::Result<()> {
        self.protocol.connect(server_addr)?;

        // Send join request
        let join_message = GameMessage::PlayerJoin(0);
        let serialized =
            bincode::serialize(&join_message).expect("Failed to serialize join message");
        self.protocol.send_reliable(serialized)?;

        Ok(())
    }

    pub fn update(&mut self) -> std::io::Result<()> {
        self.protocol.update()?;

        // Process incoming messages
        while let Some(packet) = self.protocol.buffer.incoming.pop_front() {
            let message: GameMessage =
                bincode::deserialize(&packet.data).expect("Failed to deserialize game message");

            match message {
                GameMessage::StateUpdate(new_state) => {
                    self.interpolation_buffer.push_back(new_state);
                    if self.interpolation_buffer.len() > 128 {
                        self.interpolation_buffer.pop_front();
                    }
                }
                _ => {}
            }
        }

        // Interpolate game state
        self.interpolate_state();

        Ok(())
    }

    pub fn send_input(&mut self, movement: Vector2) -> std::io::Result<()> {
        if let Some(player_id) = self.player_id {
            let input = GameMessage::PlayerInput {
                player_id,
                movement,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };

            let serialized = bincode::serialize(&input).expect("Failed to serialize input");
            self.protocol.send_reliable(serialized)?;
        }
        Ok(())
    }

    pub fn interpolate_state(&mut self) {
        if self.interpolation_buffer.len() < 2 {
            return;
        }

        // Simple linear interpolation between two states
        let old_state = self.interpolation_buffer[0].clone();
        let new_state = self.interpolation_buffer[1].clone();

        let alpha = 0.5; // Interpolation factor

        let mut interpolated_state = old_state.clone();
        for (player_id, new_player) in new_state.players.iter() {
            if let Some(old_player) = old_state.players.get(player_id) {
                let interpolated_player = PlayerState {
                    position: Vector2 {
                        x: old_player.position.x
                            + (new_player.position.x - old_player.position.x) * alpha,
                        y: old_player.position.y
                            + (new_player.position.y - old_player.position.y) * alpha,
                    },
                    ..*new_player
                };
                interpolated_state
                    .players
                    .insert(*player_id, interpolated_player);
            }
        }

        self.state = Some(interpolated_state);
    }
}

// fn main() -> std::io::Result<()> {
//     let mut client = GameClient::new("127.0.0.1:8001")?;

//     println!("Server started on 127.0.0.1:8001");

//     client.connect("127.0.0.1:8001")?;

//     loop {
//         client.update()?;

//         client.send_input(Vector2 { x: 1.0, y: 0.0 })?;

//         // Using 60fps update rate
//         std::thread::sleep(Duration::from_millis(16));
//     }
// }
