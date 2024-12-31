use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Game state structures
#[derive(Serialize, Copy, Deserialize, Clone, Debug)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub position: Vector2,
    pub velocity: Vector2,
    pub health: i32,
    pub last_update: u64, // Timestamp
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub players: HashMap<u32, PlayerState>,
    pub game_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameMessage {
    StateUpdate(GameState),
    PlayerInput {
        player_id: u32,
        movement: Vector2,
        timestamp: u64,
    },
    PlayerJoin(u32),
    PlayerLeave(u32),
    PlayerIdAssigned(u32),
}
