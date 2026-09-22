use crate::net_messages::RoomInfo;
use engine::state::GameState;

#[derive(PartialEq, Clone)]
pub enum AppMode {
    NameInput,
    Lobby,
    RoomCreating,
    RoomJoining,
    RoomHost,
    RoomGuest,
    GamePlay,
}

pub struct App {
    pub mode: AppMode,
    
    // Input buffers
    pub main_input: String,
    pub chat_input: String,
    pub is_typing_chat: bool,

    // Chat history and logs
    pub action_log: Vec<String>,
    pub chat_messages: Vec<String>,
    
    // Multiplayer Lobby State
    pub my_id: usize,
    pub is_host: bool,
    pub rooms: Vec<RoomInfo>,
    pub room_players: Vec<String>,
    
    // Live Game State
    pub game_state: Option<GameState>,
    pub connection_error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::NameInput,
            main_input: String::new(),
            chat_input: String::new(),
            is_typing_chat: false,
            action_log: Vec::new(),
            chat_messages: Vec::new(),
            my_id: 0,
            is_host: false,
            rooms: Vec::new(),
            room_players: Vec::new(),
            game_state: None,
            connection_error: None,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if self.is_typing_chat {
            self.chat_input.push(c);
        } else {
            self.main_input.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.is_typing_chat {
            self.chat_input.pop();
        } else {
            self.main_input.pop();
        }
    }

    pub fn take_input(&mut self) -> String {
        let val = self.main_input.clone();
        self.main_input.clear();
        val
    }
    
    pub fn take_chat(&mut self) -> String {
        let val = self.chat_input.clone();
        self.chat_input.clear();
        val
    }
}
