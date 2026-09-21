use crate::net_messages::{ClientMessage, RoomInfo, ServerMessage};
use engine::event::GameEvent;
use engine::player::Player;
use engine::state::GameState;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[allow(dead_code)]
struct ClientConnection {
    name: String,
    stream: TcpStream,
    room_id: Option<u32>,
}

#[allow(dead_code)]
struct Room {
    id: u32,
    name: String,
    host_id: usize,
    players: Vec<usize>,
    state: GameState,
}

#[allow(dead_code)]
struct ServerState {
    clients: HashMap<usize, ClientConnection>,
    rooms: HashMap<u32, Room>,
    next_client_id: usize,
    next_room_id: u32,
}

#[allow(dead_code)]
pub fn start_server(port: u16) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();

    // UDP Discovery Server
    if let Ok(udp_socket) = std::net::UdpSocket::bind("0.0.0.0:8081") {
        thread::spawn(move || {
            let mut buf = [0; 32];
            loop {
                if let Ok((amt, src)) = udp_socket.recv_from(&mut buf) {
                    if &buf[..amt] == b"POKER_DISCOVER" {
                        let _ = udp_socket.send_to(b"POKER_SERVER", src);
                    }
                }
            }
        });
    }

    let state = Arc::new(Mutex::new(ServerState {
        clients: HashMap::new(),
        rooms: HashMap::new(),
        next_client_id: 1,
        next_room_id: 1,
    }));

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let state_clone = Arc::clone(&state);
            thread::spawn(move || {
                handle_client(stream, state_clone);
            });
        }
    }
}

fn handle_client(stream: TcpStream, state: Arc<Mutex<ServerState>>) {
    let write_stream = stream.try_clone().unwrap();
    let client_id;
    {
        let mut s = state.lock().unwrap();
        client_id = s.next_client_id;
        s.next_client_id += 1;
        s.clients.insert(
            client_id,
            ClientConnection {
                name: format!("Guest{}", client_id),
                stream: write_stream,
                room_id: None,
            },
        );
    }

    let mut reader = std::io::BufReader::new(stream);
    let mut line = String::new();
    loop {
        match std::io::BufRead::read_line(&mut reader, &mut line) {
            Ok(0) => break,
            Ok(_) => {
                if let Ok(msg) = serde_json::from_str::<ClientMessage>(line.trim()) {
                    process_message(client_id, msg, &state);
                }
                line.clear();
            }
            Err(_) => break,
        }
    }

    let mut s = state.lock().unwrap();
    if let Some(client) = s.clients.remove(&client_id) {
        if let Some(room_id) = client.room_id {
            // Phase 1: Update the room without broadcasting
            let (is_playing, events_to_broadcast) = if let Some(room) = s.rooms.get_mut(&room_id) {
                room.players.retain(|&id| id != client_id);

                let is_playing = room.state.phase != engine::event::GamePhase::WaitingForPlayers
                    && room.state.phase != engine::event::GamePhase::Finished;

                let mut events_out = None;
                if is_playing {
                    // Try to auto-fold them
                    if let Ok(events) = room
                        .state
                        .process_action(client_id, engine::event::PlayerAction::Fold)
                    {
                        events_out = Some(events);
                    }
                    // Mark as folded and 0 chips to effectively remove them from future rounds
                    if let Some(p) = room.state.players.iter_mut().find(|p| p.id == client_id) {
                        p.is_folded = true;
                        p.chips = 0;
                    }
                }
                (is_playing, events_out)
            } else {
                (false, None)
            };

            // Phase 2: Broadcast after dropping the mutable borrow of `room`
            if is_playing {
                if let Some(events) = events_to_broadcast {
                    broadcast_game_update(&mut s, room_id, events);
                    process_bot_turns(&mut s, room_id);
                }
            } else {
                broadcast_room_state(&mut s, room_id);
            }
        }
    }
}

fn send_to_client(s: &mut ServerState, client_id: usize, msg: &ServerMessage) {
    if let Some(client) = s.clients.get_mut(&client_id) {
        let mut json = serde_json::to_string(msg).unwrap();
        json.push('\n');
        let _ = client.stream.write_all(json.as_bytes());
    }
}

fn broadcast_room_state(s: &mut ServerState, room_id: u32) {
    let (players_clone, host_id, bot_names) = if let Some(room) = s.rooms.get(&room_id) {
        let bots = room
            .state
            .players
            .iter()
            .filter(|p| p.id >= 1000)
            .map(|p| p.name.clone())
            .collect::<Vec<_>>();
        (room.players.clone(), room.host_id, bots)
    } else {
        return;
    };

    for &id in &players_clone {
        let mut players_str: Vec<String> = players_clone
            .iter()
            .map(|pid| s.clients.get(pid).unwrap().name.clone())
            .collect();
        players_str.extend(bot_names.clone());
        let msg = ServerMessage::RoomState {
            room_id,
            players: players_str,
            is_host: host_id == id,
        };
        send_to_client(s, id, &msg);
    }
}

fn process_message(client_id: usize, msg: ClientMessage, state_arc: &Arc<Mutex<ServerState>>) {
    let mut s = state_arc.lock().unwrap();
    match msg {
        ClientMessage::JoinServer { name } => {
            let name_taken = s.clients.values().any(|c| c.name == name);
            if name_taken {
                send_to_client(
                    &mut s,
                    client_id,
                    &ServerMessage::Error("Nome já está em uso.".to_string()),
                );
            } else {
                if let Some(client) = s.clients.get_mut(&client_id) {
                    client.name = name;
                }
                send_to_client(
                    &mut s,
                    client_id,
                    &ServerMessage::Welcome {
                        player_id: client_id,
                    },
                );

                let rooms = s
                    .rooms
                    .values()
                    .map(|r| RoomInfo {
                        id: r.id,
                        name: r.name.clone(),
                        player_count: r.players.len(),
                        max_players: 8,
                        has_started: r.state.phase != engine::event::GamePhase::WaitingForPlayers,
                    })
                    .collect();
                send_to_client(&mut s, client_id, &ServerMessage::LobbyState { rooms });
            }
        }
        ClientMessage::CreateRoom { room_name } => {
            let room_id = s.next_room_id;
            s.next_room_id += 1;

            let room = Room {
                id: room_id,
                name: room_name,
                host_id: client_id,
                players: vec![client_id],
                state: GameState::new(),
            };

            s.rooms.insert(room_id, room);
            if let Some(client) = s.clients.get_mut(&client_id) {
                client.room_id = Some(room_id);
            }

            broadcast_room_state(&mut s, room_id);
        }
        ClientMessage::JoinRoom { room_id } => {
            if let Some(room) = s.rooms.get_mut(&room_id) {
                if room.players.len() >= 8 {
                    send_to_client(
                        &mut s,
                        client_id,
                        &ServerMessage::Error("Sala cheia.".to_string()),
                    );
                } else {
                    room.players.push(client_id);
                    if let Some(client) = s.clients.get_mut(&client_id) {
                        client.room_id = Some(room_id);
                    }
                    broadcast_room_state(&mut s, room_id);
                }
            } else {
                send_to_client(
                    &mut s,
                    client_id,
                    &ServerMessage::Error("Sala não encontrada.".to_string()),
                );
            }
        }
        ClientMessage::AddBot => {
            if let Some(client) = s.clients.get(&client_id) {
                if let Some(room_id) = client.room_id {
                    let room = s.rooms.get_mut(&room_id).unwrap();
                    if room.host_id == client_id {
                        let bot_names =
                            ["Bot Alice", "Bot Bob", "Bot Charlie", "Bot Dave", "Bot Eve"];
                        let bot_name =
                            bot_names[room.state.players.len() % bot_names.len()].to_string();
                        let bot_id = 1000 + room.state.players.len();
                        room.state.players.push(Player::new(bot_id, bot_name, 1000));
                        broadcast_room_state(&mut s, room_id);
                    }
                }
            }
        }
        ClientMessage::StartGame => {
            if let Some(client) = s.clients.get(&client_id) {
                if let Some(room_id) = client.room_id {
                    let room_players: Vec<usize> = s.rooms.get(&room_id).unwrap().players.clone();
                    let host_id = s.rooms.get(&room_id).unwrap().host_id;

                    if host_id == client_id {
                        let mut player_names = Vec::new();
                        for &pid in &room_players {
                            player_names.push((pid, s.clients.get(&pid).unwrap().name.clone()));
                        }

                        let room = s.rooms.get_mut(&room_id).unwrap();

                        if room.state.phase == engine::event::GamePhase::Finished {
                            room.state.players.retain(|p| p.chips > 0);
                            if room.state.players.len() > 1 {
                                room.state.dealer_button =
                                    (room.state.dealer_button + 1) % room.state.players.len();
                            }
                        }

                        for (pid, name) in player_names {
                            if !room.state.players.iter().any(|p| p.id == pid) {
                                room.state.players.push(Player::new(pid, name, 1000));
                            }
                        }

                        if room.state.players.len() < 2 {
                            send_to_client(
                                &mut s,
                                client_id,
                                &ServerMessage::Error(
                                    "Not enough players with chips to start.".to_string(),
                                ),
                            );
                            return;
                        }

                        if let Ok(events) = room.state.start_game() {
                            broadcast_game_update(&mut s, room_id, events);
                            process_bot_turns(&mut s, room_id);
                        }
                    }
                }
            }
        }
        ClientMessage::Action(action) => {
            if let Some(client) = s.clients.get(&client_id) {
                if let Some(room_id) = client.room_id {
                    let room = s.rooms.get_mut(&room_id).unwrap();
                    match room.state.process_action(client_id, action) {
                        Ok(events) => {
                            broadcast_game_update(&mut s, room_id, events);
                            process_bot_turns(&mut s, room_id);
                        }
                        Err(e) => {
                            send_to_client(&mut s, client_id, &ServerMessage::Error(e.to_string()));
                        }
                    }
                }
            }
        }
    }
}

fn broadcast_game_update(s: &mut ServerState, room_id: u32, events: Vec<GameEvent>) {
    let (state_clone, players) = if let Some(room) = s.rooms.get(&room_id) {
        (room.state.clone(), room.players.clone())
    } else {
        return;
    };

    for &id in &players {
        let msg = ServerMessage::GameUpdate {
            state: state_clone.clone(),
            events: events.clone(),
            your_id: id,
        };
        send_to_client(s, id, &msg);
    }
}

fn process_bot_turns(s: &mut ServerState, room_id: u32) {
    loop {
        let (is_bot, events) = {
            let room = match s.rooms.get_mut(&room_id) {
                Some(r) => r,
                None => return,
            };

            if room.state.phase == engine::event::GamePhase::WaitingForPlayers
                || room.state.phase == engine::event::GamePhase::Finished
            {
                return;
            }

            let active_player = &room.state.players[room.state.current_turn];
            if active_player.id < 1000 {
                return; // Not a bot
            }

            let active_id = active_player.id;
            let num_opponents = room
                .state
                .players
                .iter()
                .filter(|p| p.chips > 0 && !p.is_folded && p.id != active_id)
                .count();
            let amount_to_call = room.state.current_highest_bet - active_player.current_bet;
            let active_chips = active_player.chips;

            let is_aggressive = active_id % 2 == 0;

            let win_rate = if num_opponents > 0 {
                engine::ai::calculate_win_rate(
                    &active_player.hole_cards,
                    &room.state.community_cards,
                    num_opponents,
                    1000,
                )
            } else {
                1.0
            };

            let base_win_rate = 1.0 / (num_opponents as f64 + 1.0);
            let (raise_threshold, call_threshold) = if is_aggressive {
                (base_win_rate + 0.05, base_win_rate - 0.15)
            } else {
                (base_win_rate + 0.15, base_win_rate - 0.05)
            };

            let raise_amount = if is_aggressive { 150 } else { 50 };
            let total_raise_cost = amount_to_call + raise_amount;

            let action = if win_rate > raise_threshold && active_chips >= total_raise_cost {
                engine::event::PlayerAction::Raise(raise_amount)
            } else if amount_to_call == 0 {
                engine::event::PlayerAction::Check
            } else if win_rate > call_threshold {
                engine::event::PlayerAction::Call
            } else {
                engine::event::PlayerAction::Fold
            };

            let evts = match room.state.process_action(active_id, action) {
                Ok(e) => e,
                Err(_) => {
                    let fallback = if amount_to_call > 0 {
                        engine::event::PlayerAction::Call
                    } else {
                        engine::event::PlayerAction::Check
                    };
                    match room.state.process_action(active_id, fallback) {
                        Ok(e) => e,
                        Err(_) => room
                            .state
                            .process_action(active_id, engine::event::PlayerAction::Fold)
                            .unwrap_or_default(),
                    }
                }
            };
            (true, evts)
        };

        if is_bot {
            broadcast_game_update(s, room_id, events);
        } else {
            break;
        }
    }
}
