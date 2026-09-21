use engine::event::{GameEvent, PlayerAction};
use engine::state::GameState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomInfo {
    pub id: u32,
    pub name: String,
    pub player_count: usize,
    pub max_players: usize,
    pub has_started: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    JoinServer { name: String },
    CreateRoom { room_name: String },
    JoinRoom { room_id: u32 },
    AddBot,
    StartGame,
    Action(PlayerAction),
    Chat(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Error(String),
    Chat {
        sender: String,
        message: String,
    },
    Welcome {
        player_id: usize,
    },
    LobbyState {
        rooms: Vec<RoomInfo>,
    },
    RoomState {
        room_id: u32,
        players: Vec<String>,
        is_host: bool,
    },
    GameUpdate {
        state: GameState,
        events: Vec<GameEvent>,
        your_id: usize,
    },
}
