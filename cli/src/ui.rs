use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, AppMode};
use crate::i18n::I18n;
use engine::card::Card;
use engine::state::GameState;

fn draw_cards_ascii_lines(cards: &[Card], is_board: bool) -> Vec<Line<'static>> {
    let mut lines = vec![String::new(); 5];
    let total_cards = if is_board { 5 } else { cards.len() };

    for i in 0..total_cards {
        if i < cards.len() {
            let r = cards[i].rank.to_string();
            let s = cards[i].suit.to_string();
            let pad_left = if r.len() == 2 { "" } else { " " };
            let pad_right = if r.len() == 2 { "" } else { " " };

            lines[0].push_str("┌───────┐ ");
            lines[1].push_str(&format!("│ {}{}    │ ", r, pad_left));
            lines[2].push_str(&format!("│   {}   │ ", s));
            lines[3].push_str(&format!("│    {}{} │ ", pad_right, r));
            lines[4].push_str("└───────┘ ");
        } else if is_board {
            lines[0].push_str("┌───────┐ ");
            lines[1].push_str("│ ░░░░░ │ ");
            lines[2].push_str("│ ░░░░░ │ ");
            lines[3].push_str("│ ░░░░░ │ ");
            lines[4].push_str("└───────┘ ");
        }
    }

    lines.into_iter().map(Line::from).collect()
}

pub fn render_ratatui(f: &mut ratatui::Frame, app: &App, i18n: &I18n) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());

    let left_chunk = chunks[0];
    let right_chunk = chunks[1];

    let block_style = Style::default().fg(Color::White);
    let border_type = BorderType::Rounded;

    // --- LEFT PANE (Main Content) ---
    let main_block = Block::default()
        .title(" TEXAS HOLD'EM CLI ")
        .borders(Borders::ALL)
        .border_type(border_type)
        .style(block_style);

    let mut main_text = Vec::new();

    if let Some(err) = &app.connection_error {
        main_text.push(Line::from(vec![Span::styled(format!("ERROR: {}", err), Style::default().fg(Color::Red))]));
        main_text.push(Line::from(""));
    }

    match app.mode {
        AppMode::NameInput => {
            main_text.push(Line::from("Welcome! Please enter your name:"));
            main_text.push(Line::from(format!("> {}", app.main_input)));
        }
        AppMode::Lobby => {
            main_text.push(Line::from("=== MULTIPLAYER LOBBY ==="));
            if app.rooms.is_empty() {
                main_text.push(Line::from("No rooms found."));
            } else {
                for r in &app.rooms {
                    let status = if r.has_started { "IN PROGRESS" } else { "WAITING" };
                    main_text.push(Line::from(format!(" [{}] {} ({}/8) - {}", r.id, r.name, r.player_count, status)));
                }
            }
            main_text.push(Line::from(""));
            main_text.push(Line::from("[1] Create Room | [2] Join Room"));
            main_text.push(Line::from(format!("> {}", app.main_input)));
        }
        AppMode::RoomCreating => {
            main_text.push(Line::from("Enter a name for your room:"));
            main_text.push(Line::from(format!("> {}", app.main_input)));
        }
        AppMode::RoomJoining => {
            main_text.push(Line::from("Enter the Room ID to join:"));
            main_text.push(Line::from(format!("> {}", app.main_input)));
        }
        AppMode::RoomHost | AppMode::RoomGuest => {
            main_text.push(Line::from("=== WAITING ROOM ==="));
            main_text.push(Line::from("Players connected:"));
            for name in &app.room_players {
                main_text.push(Line::from(format!(" - {}", name)));
            }
            if app.mode == AppMode::RoomHost {
                main_text.push(Line::from(""));
                main_text.push(Line::from("Press [s] to Start Game, [b] to Add Bot"));
                main_text.push(Line::from(format!("> {}", app.main_input)));
            } else {
                main_text.push(Line::from(""));
                main_text.push(Line::from("Waiting for host to start the game..."));
            }
        }
        AppMode::GamePlay => {
            if let Some(game) = &app.game_state {
                main_text.push(Line::from(format!(
                    "{} {} | {} ${} | {} ${}",
                    i18n.t("phase"), i18n.t_phase(&game.phase),
                    i18n.t("pot"), game.pot,
                    i18n.t("highest_bet"), game.current_highest_bet
                )));
                main_text.push(Line::from(""));

                main_text.push(Line::from(i18n.t("board_cards")));
                for line in draw_cards_ascii_lines(&game.community_cards, true) {
                    main_text.push(line);
                }
                main_text.push(Line::from(""));

                main_text.push(Line::from(i18n.t("players")));
                let header = i18n.t("table_header").replace("{}", "=>");
                main_text.push(Line::from(header));

                let active_count = game.players.iter().filter(|p| p.chips > 0 || p.is_all_in).count();
                let true_sb = if active_count == 2 { game.dealer_button } else { game.next_active_player(game.dealer_button) };
                let true_bb = if active_count == 2 { game.next_active_player(game.dealer_button) } else { game.next_active_player(true_sb) };

                for (i, p) in game.players.iter().enumerate() {
                    let is_dealer = game.dealer_button == i;
                    let is_sb = true_sb == i;
                    let is_bb = true_bb == i;
                    
                    let role = if is_dealer { "[D] " } else if is_sb { "[SB]" } else if is_bb { "[BB]" } else { "    " };
                    let active_token = if game.current_turn == i { "=>" } else { "  " };

                    let status = if p.chips == 0 && !p.is_all_in {
                        i18n.t("eliminated")
                    } else if p.is_folded {
                        i18n.t("folded")
                    } else if p.is_all_in {
                        i18n.t("all_in")
                    } else {
                        ""
                    };

                    let mut name = p.name.clone();
                    if name.chars().count() > 15 {
                        name = name.chars().take(12).collect::<String>();
                        name.push_str("...");
                    }
                    
                    main_text.push(Line::from(format!(
                        "| {} | {} | {:<15} | ${:<6} | ${:<7} | {:<10} |",
                        active_token, role, name, p.chips, p.current_bet, status
                    )));
                }
                main_text.push(Line::from(""));

                if let Some(me) = game.players.iter().find(|p| p.id == app.my_id) {
                    main_text.push(Line::from("Suas Cartas:"));
                    if !me.hole_cards.is_empty() {
                        for line in draw_cards_ascii_lines(&me.hole_cards, false) {
                            main_text.push(line);
                        }
                    }

                    if game.current_turn == app.my_id && game.phase != engine::event::GamePhase::WaitingForPlayers && game.phase != engine::event::GamePhase::Finished {
                        let call_amt = game.current_highest_bet - me.current_bet;
                        let menu_str = if call_amt == 0 {
                            format!("{}: [1] {} | [2] {} | [3] {}", i18n.t("menu_actions"), i18n.t("menu_fold"), i18n.t("menu_check"), i18n.t("menu_raise"))
                        } else {
                            format!("{}: [1] {} | [2] {} (${}) | [3] {}", i18n.t("menu_actions"), i18n.t("menu_fold"), i18n.t("menu_call"), call_amt, i18n.t("menu_raise"))
                        };
                        main_text.push(Line::from(menu_str));
                        main_text.push(Line::from(format!("{}> {}", i18n.t("action_prompt"), app.main_input)));
                    }
                }
            }
        }
    }

    let p = Paragraph::new(main_text).block(main_block);
    f.render_widget(p, left_chunk);


    // --- RIGHT PANE (Chat & Logs) ---
    let chat_block = Block::default()
        .title(" Logs & Chat (Tab) ")
        .borders(Borders::ALL)
        .border_type(border_type)
        .style(if app.is_typing_chat { Style::default().fg(Color::Yellow) } else { block_style });

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(right_chunk);

    let log_items: Vec<ListItem> = app.action_log.iter()
        .skip(app.action_log.len().saturating_sub(50)) // Tail last 50
        .map(|msg| ListItem::new(Line::from(msg.clone())))
        .collect();

    let logs_list = List::new(log_items).block(chat_block);
    f.render_widget(logs_list, right_layout[0]);

    let chat_input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .title(" Escrever ");

    let chat_input_text = Paragraph::new(format!("> {}", app.chat_input)).block(chat_input_block);
    f.render_widget(chat_input_text, right_layout[1]);
}
