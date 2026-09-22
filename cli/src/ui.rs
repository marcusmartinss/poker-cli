use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, AppMode};
use crate::i18n::I18n;
use engine::card::Card;

fn draw_cards_ascii_lines(cards: &[Card], is_board: bool) -> Vec<Line<'static>> {
    let mut spans_lines: Vec<Vec<Span<'static>>> = vec![vec![], vec![], vec![], vec![], vec![]];
    let total_cards = if is_board { 5 } else { cards.len() };

    for i in 0..total_cards {
        if i < cards.len() {
            let r = cards[i].rank.to_string();
            let s = cards[i].suit.to_string();
            let pad_left = if r.len() == 2 { "" } else { " " };
            let pad_right = if r.len() == 2 { "" } else { " " };

            let is_red = cards[i].suit == engine::card::Suit::Hearts || cards[i].suit == engine::card::Suit::Diamonds;
            let style = if is_red { Style::default().fg(Color::Red) } else { Style::default() };

            spans_lines[0].push(Span::styled("┌───────┐ ", style));
            spans_lines[1].push(Span::styled(format!("│ {}{}    │ ", r, pad_left), style));
            spans_lines[2].push(Span::styled(format!("│   {}   │ ", s), style));
            spans_lines[3].push(Span::styled(format!("│    {}{} │ ", pad_right, r), style));
            spans_lines[4].push(Span::styled("└───────┘ ", style));
        } else if is_board {
            let style = Style::default().fg(Color::DarkGray);
            spans_lines[0].push(Span::styled("┌───────┐ ", style));
            spans_lines[1].push(Span::styled("│ ░░░░░ │ ", style));
            spans_lines[2].push(Span::styled("│ ░░░░░ │ ", style));
            spans_lines[3].push(Span::styled("│ ░░░░░ │ ", style));
            spans_lines[4].push(Span::styled("└───────┘ ", style));
        }
    }

    spans_lines.into_iter().map(Line::from).collect()
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
            for (name, ready) in &app.room_players {
                let status = if *ready { "(PRONTO)" } else { "(Aguardando...)" };
                main_text.push(Line::from(format!(" - {} {}", name, status)));
            }
            main_text.push(Line::from(""));
            if app.mode == AppMode::RoomHost {
                main_text.push(Line::from("Comandos: [s] Iniciar | [b] Add Bot | [r] Alternar Pronto"));
            } else {
                main_text.push(Line::from("Comandos: [r] Alternar Pronto (Aguardando Host iniciar)"));
            }
            main_text.push(Line::from(format!("> {}", app.main_input)));
        }
        AppMode::GamePlay | AppMode::GamePlayRaising => {
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
                    let mut title = vec![Span::raw("Suas Cartas:")];
                    let mut all_cards = me.hole_cards.clone();
                    all_cards.extend(game.community_cards.clone());
                    
                    if all_cards.len() == 2 {
                        let rank_str = if all_cards[0].rank == all_cards[1].rank {
                            i18n.t_hand("Pair")
                        } else {
                            i18n.t_hand("HighCard")
                        };
                        title.push(Span::styled(format!(" ({})", rank_str), ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)));
                    } else if all_cards.len() >= 5 {
                        if let Ok(rank) = engine::evaluator::evaluate(&all_cards) {
                            let rank_str = i18n.t_hand(&format!("{:?}", rank));
                            title.push(Span::styled(format!(" ({})", rank_str), ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)));
                        }
                    }
                    main_text.push(Line::from(title));
                    if !me.hole_cards.is_empty() {
                        for line in draw_cards_ascii_lines(&me.hole_cards, false) {
                            main_text.push(line);
                        }
                    }

                    if game.phase == engine::event::GamePhase::Finished {
                        main_text.push(Line::from(""));
                        
                        // Busca quem ganhou no log e exibe no meio da tela bem grande
                        for log in app.action_log.iter().rev().take(15) {
                            if log.starts_with("***") {
                                main_text.push(Line::from(Span::styled(log.clone(), ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD))));
                            }
                        }
                        main_text.push(Line::from(""));

                        if app.is_host {
                            main_text.push(Line::from(Span::styled("Pressione [Enter] para iniciar a próxima mão.", ratatui::style::Style::default().fg(ratatui::style::Color::Green))));
                        } else {
                            main_text.push(Line::from(Span::styled("Aguardando o host iniciar a próxima mão...", ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))));
                        }
                    }

                    if game.phase != engine::event::GamePhase::WaitingForPlayers && game.phase != engine::event::GamePhase::Finished {
                        let elapsed = app.turn_start_time.elapsed().as_secs();
                        let remaining = 15_u64.saturating_sub(elapsed);
                        let color = if remaining <= 5 { ratatui::style::Color::Red } else { ratatui::style::Color::Green };
                        
                        if game.players[game.current_turn].id == app.my_id {
                            main_text.push(Line::from(Span::styled(format!("SUA VEZ! Tempo restante: {}s", remaining), ratatui::style::Style::default().fg(color).add_modifier(ratatui::style::Modifier::BOLD))));
                            main_text.push(Line::from(""));
                            
                            if app.mode == AppMode::GamePlayRaising {
                                main_text.push(Line::from(format!("Valor para aumentar (min: {}, ou 'min'/'all'):", game.min_raise)));
                                main_text.push(Line::from(format!("> {}", app.main_input)));
                            } else {
                                let call_amt = game.current_highest_bet - me.current_bet;
                                let menu_str = if call_amt == 0 {
                                    format!("{}: [1] {} | [2] {} | [3] {}", i18n.t("menu_actions"), i18n.t("menu_fold"), i18n.t("menu_check"), i18n.t("menu_raise"))
                                } else {
                                    format!("{}: [1] {} | [2] {} (${}) | [3] {}", i18n.t("menu_actions"), i18n.t("menu_fold"), i18n.t("menu_call"), call_amt, i18n.t("menu_raise"))
                                };
                                main_text.push(Line::from(menu_str));
                                main_text.push(Line::from(format!("{}> {}", i18n.t("action_prompt"), app.main_input)));
                            }
                        } else if game.current_turn < game.players.len() {
                            let current_player_name = &game.players[game.current_turn].name;
                            main_text.push(Line::from(""));
                            main_text.push(Line::from(Span::styled(format!("Turno de {}... ({}s)", current_player_name, remaining), ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray))));
                        }
                    }
                }
            }
        }
    }

    let p = Paragraph::new(main_text).block(main_block);
    f.render_widget(p, left_chunk);


    // --- RIGHT PANE (Logs & Chat) ---
    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Top: Game Logs
            Constraint::Min(0),         // Bottom: Chat history
            Constraint::Length(3)       // Bottom-most: Chat input
        ])
        .split(right_chunk);

    // 1. Logs
    let logs_block = Block::default()
        .title(" Ações (Logs) ")
        .borders(Borders::ALL)
        .border_type(border_type)
        .style(block_style);
        
    let log_width = right_layout[0].width.saturating_sub(2) as usize;
    let log_height = right_layout[0].height.saturating_sub(2) as usize;
    let mut log_items = Vec::new();
    for msg in app.action_log.iter().skip(app.action_log.len().saturating_sub(50)) {
        let chars: Vec<char> = msg.chars().collect();
        if chars.is_empty() {
            log_items.push(ListItem::new(Line::from("")));
        } else {
            for chunk in chars.chunks(log_width.max(1)) {
                let chunk_str: String = chunk.iter().collect();
                log_items.push(ListItem::new(Line::from(chunk_str)));
            }
        }
    }
    let visible_logs: Vec<ListItem> = log_items.into_iter().rev().take(log_height).rev().collect();
    let logs_list = List::new(visible_logs).block(logs_block);
    f.render_widget(logs_list, right_layout[0]);

    // 2. Chat Messages
    let chat_block = Block::default()
        .title(" Chat Global ")
        .borders(Borders::ALL)
        .border_type(border_type)
        .style(block_style);
        
    let chat_width = right_layout[1].width.saturating_sub(2) as usize;
    let chat_height = right_layout[1].height.saturating_sub(2) as usize;
    let mut chat_items = Vec::new();
    for msg in app.chat_messages.iter().skip(app.chat_messages.len().saturating_sub(50)) {
        let chars: Vec<char> = msg.chars().collect();
        if chars.is_empty() {
            chat_items.push(ListItem::new(Line::from("")));
        } else {
            for chunk in chars.chunks(chat_width.max(1)) {
                let chunk_str: String = chunk.iter().collect();
                chat_items.push(ListItem::new(Line::from(chunk_str)));
            }
        }
    }
    let visible_chat: Vec<ListItem> = chat_items.into_iter().rev().take(chat_height).rev().collect();
    let chat_list = List::new(visible_chat).block(chat_block);
    f.render_widget(chat_list, right_layout[1]);

    // 3. Chat Input
    let chat_input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .title(" Mensagem (Pressione Tab para focar) ")
        .style(if app.is_typing_chat { Style::default().fg(Color::Yellow) } else { block_style });

    let chat_input_text = Paragraph::new(format!("> {}", app.chat_input)).block(chat_input_block);
    f.render_widget(chat_input_text, right_layout[2]);
}
