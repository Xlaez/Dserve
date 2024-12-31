use bincode;
use std::{collections::HashMap, time::Duration};

use dserve::{
    definitions::NetworkProtocol,
    game_server::{
        client::GameClient,
        types::{GameMessage, GameState, PlayerState, Vector2},
    },
};

struct GameServer {
    protocol: NetworkProtocol,
    state: GameState,
    clients: HashMap<String, u32>,
    next_player_id: u32,
}

impl GameServer {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        Ok(Self {
            protocol: NetworkProtocol::new(addr)?,
            state: GameState {
                players: HashMap::new(),
                game_time: 0,
            },
            clients: HashMap::new(),
            next_player_id: 1,
        })
    }

    pub fn update(&mut self) -> std::io::Result<()> {
        // Update network
        self.protocol.update()?;

        // Process incoming messages
        while let Some(packet) = self.protocol.buffer.incoming.pop_front() {
            print!("packet: {:?}", packet);
            let message: GameMessage =
                bincode::deserialize(&packet.data).expect("Failed to deserialize game message");

            match message {
                GameMessage::PlayerInput {
                    player_id,
                    movement,
                    timestamp,
                } => {
                    if let Some(player) = self.state.players.get_mut(&player_id) {
                        player.velocity = movement;
                        player.last_update = timestamp;
                    }
                }
                GameMessage::PlayerJoin(client_addr) => {
                    print!("client_addr: {:?}", client_addr);
                    let player_id = self.next_player_id;
                    self.next_player_id += 1;

                    let new_player = PlayerState {
                        position: Vector2 { x: 0.0, y: 0.0 },
                        velocity: Vector2 { x: 0.0, y: 0.0 },
                        health: 100,
                        last_update: self.state.game_time,
                    };

                    self.state.players.insert(player_id, new_player);
                }
                GameMessage::PlayerLeave(player_id) => {
                    self.state.players.remove(&player_id);
                }
                _ => {}
            }
        }

        // Update game state
        self.update_game_state();

        // Broadcast state to all clients
        let state_update = GameMessage::StateUpdate(self.state.clone());
        let serialized = bincode::serialize(&state_update).expect("Failed to serialize game state");
        self.protocol.send_reliable(serialized)?;

        Ok(())
    }

    pub fn update_game_state(&mut self) {
        self.state.game_time += 1;

        // Update player positions based on velocity
        for player in self.state.players.values_mut() {
            player.position.x += player.velocity.x;
            player.position.y += player.velocity.y;
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut server = GameServer::new("127.0.0.1:8000")?;

    println!("Server started on 127.0.0.1:8000");

    let mut client = GameClient::new("127.0.0.1:8001")?;

    println!("Client server started on 127.0.0.1:8001 attempting to connect to 127.0.0.1:8000");

    client.connect("127.0.0.1:8000")?;

    loop {
        server.update()?;
        client.update()?;

        client.send_input(Vector2 { x: 1.0, y: 0.0 })?;

        // Using 60fps update rate
        std::thread::sleep(Duration::from_millis(16));
    }
}
