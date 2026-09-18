use std::net::TcpStream;
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::net_messages::{ClientMessage, ServerMessage};
use engine::event::PlayerAction;
use crate::i18n::I18n;
use engine::state::GameState;

enum InputMode {
    Name,
    Lobby,
    RoomHost,
    RoomGuest,
    GamePlay,
}

pub fn start_client(ip: &str, port: u16, i18n: &I18n) {
    let mut stream = match TcpStream::connect(format!("{}:{}", ip, port)) {
        Ok(s) => s,
        Err(_) => {
            println!("Failed to connect to {}:{}", ip, port);
            return;
        }
    };
    
    let mut write_stream = stream.try_clone().unwrap();
    
    let (server_tx, server_rx) = mpsc::channel();
    let (input_tx, input_rx) = mpsc::channel();
    
    // Server Reader Thread
    thread::spawn(move || {
        let mut buffer = [0; 8192];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let msg_str = String::from_utf8_lossy(&buffer[..n]);
                    for line in msg_str.lines() {
                        if line.trim().is_empty() { continue; }
                        if let Ok(msg) = serde_json::from_str::<ServerMessage>(line) {
                            let _ = server_tx.send(msg);
                        }
                    }
                }
                Err(_) => break,
            }
        }
        println!("\nDisconnected from server.");
        std::process::exit(0);
    });
    
    // Stdin Reader Thread
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                let _ = input_tx.send(input.trim().to_string());
            }
        }
    });

    let mut mode = InputMode::Name;
    
    // Helper closure to send msg
    let mut send_msg = |msg: ClientMessage| {
        let mut json = serde_json::to_string(&msg).unwrap();
        json.push('\n');
        let _ = write_stream.write_all(json.as_bytes());
    };
    
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("--- Multiplayer Poker ---");
    print!("Enter your player name: ");
    let _ = io::stdout().flush();

    let mut action_log = Vec::new();
    let mut game_state_opt: Option<GameState> = None;
    let mut my_id = 0;
    
    loop {
        // Handle User Input
        if let Ok(input) = input_rx.try_recv() {
            match mode {
                InputMode::Name => {
                    if !input.is_empty() {
                        send_msg(ClientMessage::JoinServer { name: input.clone() });
                    }
                }
                InputMode::Lobby => {
                    let upper = input.to_uppercase();
                    if upper.starts_with('C') {
                        let parts: Vec<&str> = input.splitn(2, ' ').collect();
                        let room_name = if parts.len() > 1 { parts[1].to_string() } else { "My Room".to_string() };
                        send_msg(ClientMessage::CreateRoom { room_name });
                    } else if let Ok(room_id) = input.parse::<u32>() {
                        send_msg(ClientMessage::JoinRoom { room_id });
                    }
                }
                InputMode::RoomHost => {
                    match input.as_str() {
                        "1" => send_msg(ClientMessage::AddBot),
                        "2" => send_msg(ClientMessage::StartGame),
                        _ => println!("Invalid option."),
                    }
                }
                InputMode::RoomGuest => {
                    // Nothing to do
                }
                InputMode::GamePlay => {
                    if let Some(game) = &game_state_opt {
                        if let Some(me) = game.players.iter().find(|p| p.id == my_id) {
                            if game.current_turn < game.players.len() && game.players[game.current_turn].id == my_id {
                                let action = match input.as_str() {
                                    "1" => Some(PlayerAction::Fold),
                                    "2" => Some(PlayerAction::Check),
                                    "3" => Some(PlayerAction::Call),
                                    "4" => {
                                        print!("{}", i18n.t("raise_prompt"));
                                        let _ = io::stdout().flush();
                                        // Blocking read is ok here since it's just the amount prompt
                                        let mut amt_input = String::new();
                                        let _ = io::stdin().read_line(&mut amt_input);
                                        let amt_trim = amt_input.trim().to_lowercase();
                                        let call_amt = game.current_highest_bet - me.current_bet;
                                        
                                        if amt_trim == "all" {
                                            if me.chips <= call_amt {
                                                Some(PlayerAction::Call)
                                            } else {
                                                Some(PlayerAction::Raise(me.chips - call_amt))
                                            }
                                        } else if let Ok(amt) = amt_trim.parse::<u32>() {
                                            Some(PlayerAction::Raise(amt))
                                        } else {
                                            println!("Invalid amount.");
                                            None
                                        }
                                    },
                                    _ => {
                                        println!("Unknown command.");
                                        None
                                    }
                                };
                                
                                if let Some(a) = action {
                                    send_msg(ClientMessage::Action(a));
                                } else {
                                    print!("{}\n=> ", i18n.t("actions_menu"));
                                    let _ = io::stdout().flush();
                                }
                            }
                        }
                    }
                }
            }
        }

        // Handle Server Messages
        if let Ok(msg) = server_rx.try_recv() {
            match msg {
                ServerMessage::Error(e) => {
                    println!("[ERROR] {}", e);
                    if matches!(mode, InputMode::Name) {
                        print!("Try another name: ");
                        let _ = io::stdout().flush();
                    }
                }
                ServerMessage::Welcome { player_id } => {
                    my_id = player_id;
                    mode = InputMode::Lobby;
                }
                ServerMessage::LobbyState { rooms } => {
                    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                    println!("=== LOBBY ===");
                    if rooms.is_empty() {
                        println!("No rooms available.");
                    } else {
                        println!("Available Rooms:");
                        for r in rooms {
                            let status = if r.has_started { "(In Game)" } else { "(Waiting)" };
                            println!(" [{}] {} - {}/{} players {}", r.id, r.name, r.player_count, r.max_players, status);
                        }
                    }
                    println!("\nCommands:");
                    println!(" 'C <Room Name>' - Create Room");
                    println!(" '<Room ID>'     - Join Room");
                    print!("\n=> ");
                    let _ = io::stdout().flush();
                }
                ServerMessage::RoomState { room_id, players, is_host } => {
                    mode = if is_host { InputMode::RoomHost } else { InputMode::RoomGuest };
                    
                    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                    println!("=== ROOM {} ===", room_id);
                    println!("Players:");
                    for (i, p) in players.iter().enumerate() {
                        println!(" {}. {}", i+1, p);
                    }
                    
                    if is_host {
                        println!("\nHost Menu:");
                        println!(" [1] Add Bot");
                        println!(" [2] Start Game");
                        print!("\n=> ");
                    } else {
                        println!("\nWaiting for host to start the game...");
                    }
                    let _ = io::stdout().flush();
                }
                ServerMessage::GameUpdate { state, events, your_id } => {
                    mode = InputMode::GamePlay;
                    my_id = your_id;
                    game_state_opt = Some(state.clone());
                    
                    // Render events
                    crate::process_events(&events, &state, &mut action_log, i18n);
                    
                    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                    crate::ui::render_table(i18n, &state);
                    
                    if state.phase == engine::event::GamePhase::Showdown || state.phase == engine::event::GamePhase::Finished {
                        crate::ui::render_showdown(i18n, &state);
                    }
                    
                    println!("  [ {} ]", i18n.t("final_actions"));
                    let recent = if action_log.len() > 8 { &action_log[action_log.len()-8..] } else { &action_log[..] };
                    for msg in recent {
                        println!("   > {}", msg);
                    }
                    println!("---------------------------------------------------------\n");
                    
                    if state.phase == engine::event::GamePhase::Finished {
                        println!("Hand Finished. Waiting for Host/Server to continue...");
                    } else if state.phase != engine::event::GamePhase::Showdown {
                        // Display my cards if I'm not folded and not finished
                        if let Some(me) = state.players.iter().find(|p| p.id == my_id) {
                            if me.hole_cards.len() == 2 {
                                let mut my_cards = state.community_cards.clone();
                                my_cards.extend(me.hole_cards.clone());
                                let raw_hand = if my_cards.len() == 2 {
                                    if my_cards[0].rank == my_cards[1].rank { "Pair".to_string() } else { "HighCard".to_string() }
                                } else {
                                    match engine::evaluator::evaluate(&my_cards) {
                                        Ok(rank) => format!("{:?}", rank),
                                        Err(_) => "".to_string(),
                                    }
                                };
                                println!("  {} ( {} )", i18n.t("your_cards"), i18n.t_hand(&raw_hand));
                                crate::ui::draw_cards_ascii(&me.hole_cards, false);
                                println!();
                            }
                        }

                        // Check if it's my turn
                        if state.current_turn < state.players.len() && state.players[state.current_turn].id == my_id {
                            println!("{}", i18n.t("your_turn"));
                            println!("{}", i18n.t("actions_menu"));
                            print!("{}", i18n.t("action_prompt"));
                            let _ = io::stdout().flush();
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}
