use crate::app::{App, AppMode};
use crate::i18n::I18n;
use crate::net_messages::{ClientMessage, ServerMessage};
use crate::tui::Tui;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use engine::event::PlayerAction;

pub fn start_client(ip: &str, port: u16, i18n: &I18n) {
    let stream = match TcpStream::connect(format!("{}:{}", ip, port)) {
        Ok(s) => s,
        Err(_) => {
            println!("Failed to connect to {}:{}", ip, port);
            return;
        }
    };

    stream.set_nonblocking(true).unwrap();
    let mut write_stream = stream.try_clone().unwrap();

    let (server_tx, server_rx) = mpsc::channel();

    // Server Reader Thread
    thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stream);
        let mut line = String::new();
        loop {
            match std::io::BufRead::read_line(&mut reader, &mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(msg) = serde_json::from_str::<ServerMessage>(line.trim()) {
                        let _ = server_tx.send(msg);
                    }
                    line.clear();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });

    let mut app = App::new();
    let mut tui = Tui::init().unwrap();

    let mut send_msg = |msg: ClientMessage| {
        let mut json = serde_json::to_string(&msg).unwrap();
        json.push('\n');
        let _ = write_stream.write_all(json.as_bytes());
    };

    loop {
        // 1. Render UI
        let _ = tui.terminal.draw(|f| ui::render_ratatui(f, &app, i18n));

        // 2. Poll Input
        if event::poll(Duration::from_millis(16)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                match key.code {
                    KeyCode::Esc => {
                        if app.is_typing_chat {
                            app.is_typing_chat = false;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Tab => {
                        app.is_typing_chat = !app.is_typing_chat;
                    }
                    KeyCode::Char(c) => app.handle_char(c),
                    KeyCode::Backspace => app.handle_backspace(),
                    KeyCode::Enter => {
                        if app.is_typing_chat {
                            let msg = app.take_chat();
                            if !msg.is_empty() {
                                if msg.starts_with('/') {
                                    let cmd = msg.trim_start_matches('/');
                                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                                    match parts.first().copied() {
                                        Some("quit") | Some("leave") | Some("sair") => {
                                            break;
                                        }
                                        Some("help") | Some("ajuda") => {
                                            app.chat_messages.push("[Sistema] Comandos disponíveis: /quit (Sair), /kick <nome>".to_string());
                                        }
                                        Some("kick") => {
                                            if parts.len() > 1 {
                                                let target = parts[1..].join(" ");
                                                send_msg(ClientMessage::Kick(target));
                                            } else {
                                                app.chat_messages.push("[Sistema] Uso: /kick <nome_do_jogador>".to_string());
                                            }
                                        }
                                        _ => {
                                            app.chat_messages.push(format!("[Sistema] Comando desconhecido: {}", msg));
                                        }
                                    }
                                } else {
                                    send_msg(ClientMessage::Chat(msg));
                                }
                            }
                        } else {
                            let input = app.take_input();
                            let input_trim = input.trim();
                            match app.mode {
                                AppMode::NameInput => {
                                    send_msg(ClientMessage::JoinServer { name: input_trim.to_string() });
                                }
                                AppMode::Lobby => {
                                    if input_trim == "1" {
                                        app.mode = AppMode::RoomCreating;
                                    } else if input_trim == "2" {
                                        app.mode = AppMode::RoomJoining;
                                    }
                                }
                                AppMode::RoomCreating => {
                                    send_msg(ClientMessage::CreateRoom { room_name: input_trim.to_string() });
                                }
                                AppMode::RoomJoining => {
                                    if let Ok(id) = input_trim.parse::<u32>() {
                                        send_msg(ClientMessage::JoinRoom { room_id: id });
                                    }
                                }
                                AppMode::RoomHost | AppMode::RoomGuest => {
                                    if input_trim == "s" && app.mode == AppMode::RoomHost {
                                        send_msg(ClientMessage::StartGame);
                                    } else if input_trim == "b" && app.mode == AppMode::RoomHost {
                                        send_msg(ClientMessage::AddBot);
                                    } else if input_trim == "r" {
                                        send_msg(ClientMessage::ToggleReady);
                                    }
                                }
                                AppMode::GamePlay => {
                                    if let Some(game) = &app.game_state {
                                        if game.phase == engine::event::GamePhase::Finished {
                                            if app.is_host {
                                                send_msg(ClientMessage::StartGame);
                                            }
                                        } else if game.players[game.current_turn].id == app.my_id {
                                            if let Some(me) = game.players.iter().find(|p| p.id == app.my_id) {
                                                let call_amt = game.current_highest_bet - me.current_bet;
                                                let active_chips = me.chips;
                                                let amt_trim = input_trim.to_lowercase();
                                                
                                                let action_opt = if amt_trim == "1" {
                                                    Some(PlayerAction::Fold)
                                                } else if amt_trim == "2" {
                                                    if call_amt == 0 { Some(PlayerAction::Check) } else { Some(PlayerAction::Call) }
                                                } else if amt_trim == "3" {
                                                    app.mode = AppMode::GamePlayRaising;
                                                    None
                                                } else if amt_trim == "all" {
                                                    Some(PlayerAction::Raise(active_chips.saturating_sub(call_amt)))
                                                } else if amt_trim == "min" {
                                                    let raise_amt = std::cmp::min(game.min_raise, active_chips.saturating_sub(call_amt));
                                                    Some(PlayerAction::Raise(raise_amt))
                                                } else {
                                                    None
                                                };
                                                
                                                if let Some(action) = action_opt {
                                                    send_msg(ClientMessage::Action(action));
                                                }
                                            }
                                        }
                                    }
                                }
                                AppMode::GamePlayRaising => {
                                    if let Some(game) = &app.game_state {
                                        if let Ok(amt) = input_trim.parse::<u32>() {
                                            send_msg(ClientMessage::Action(PlayerAction::Raise(amt)));
                                        } else if input_trim.to_lowercase() == "all" {
                                            if let Some(me) = game.players.iter().find(|p| p.id == app.my_id) {
                                                let call_amt = game.current_highest_bet - me.current_bet;
                                                send_msg(ClientMessage::Action(PlayerAction::Raise(me.chips.saturating_sub(call_amt))));
                                            }
                                        }
                                        app.mode = AppMode::GamePlay;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // 3. Poll Server Messages
        while let Ok(msg) = server_rx.try_recv() {
            match msg {
                ServerMessage::Error(e) => {
                    app.connection_error = Some(e.clone());
                    app.connection_error_time = Some(std::time::Instant::now());
                    if app.mode == AppMode::RoomCreating || app.mode == AppMode::RoomJoining {
                        app.mode = AppMode::Lobby;
                    }
                    if e == "Você foi removido da sala pelo Host." {
                        app.mode = AppMode::Lobby;
                        app.game_state = None;
                    }
                }
                ServerMessage::Welcome { player_id } => {
                    app.my_id = player_id;
                    app.mode = AppMode::Lobby;
                }
                ServerMessage::LobbyState { rooms } => {
                    app.rooms = rooms;
                }
                ServerMessage::RoomState { room_id: _, players, is_host } => {
                    app.is_host = is_host;
                    app.room_players = players;
                    app.mode = if is_host { AppMode::RoomHost } else { AppMode::RoomGuest };
                }
                ServerMessage::GameUpdate { state, events, your_id: _ } => {
                    app.mode = AppMode::GamePlay;
                    crate::event_logger::process_events(&events, &state, &mut app.action_log, i18n);
                    
                    if let Some(me) = state.players.iter().find(|p| p.id == app.my_id) {
                        if me.chips == 0 && !me.is_all_in {
                            app.is_typing_chat = true;
                        }
                    }
                    
                    app.game_state = Some(state);
                }
                ServerMessage::Chat { sender, message } => {
                    app.chat_messages.push(format!("{}: {}", sender, message));
                }
            }
        }
    }
}
