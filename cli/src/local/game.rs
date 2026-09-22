use crate::event_logger::process_events;
use crate::i18n::I18n;
use crate::app::{App, AppMode};
use crate::ui;
use crate::tui::Tui;
use engine::event::{GamePhase, PlayerAction};
use engine::player::Player;
use engine::state::GameState;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

pub fn play_local(i18n: &I18n) {
    let mut game = GameState::new();

    use std::time::{SystemTime, UNIX_EPOCH};
    let is_third_aggressive = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % 2
        == 0;
    let bot3_id = if is_third_aggressive { 4 } else { 3 }; // 4 = Aggressive, 3 = Conservative

    game.players.push(Player::new(0, i18n.t("human_name").to_string(), 10000));
    game.players.push(Player::new(1, "Bot Alice".to_string(), 10000));
    game.players.push(Player::new(2, "Bot Bob".to_string(), 10000));
    game.players.push(Player::new(bot3_id, "Bot Charlie".to_string(), 10000));

    let mut app = App::new();
    app.mode = AppMode::GamePlay;
    app.my_id = 0;

    let mut tui = Tui::init().unwrap();

    // Start first hand
    match game.start_game() {
        Ok(events) => process_events(&events, &game, &mut app.action_log, i18n),
        Err(_) => return,
    }
    app.game_state = Some(game.clone());

    loop {
        let _ = tui.terminal.draw(|f| ui::render_ratatui(f, &app, i18n));

        let mut game_state = app.game_state.take().unwrap();

        if game_state.phase == GamePhase::Finished {
            if event::poll(Duration::from_millis(16)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }
                    if key.code == KeyCode::Enter {
                        if game_state.players.iter().filter(|p| p.chips > 0).count() <= 1 {
                            break; // Game over
                        }
                        game_state.dealer_button = game_state.next_active_player(game_state.dealer_button);
                        app.action_log.clear();
                        if let Ok(events) = game_state.start_game() {
                            process_events(&events, &game_state, &mut app.action_log, i18n);
                        }
                    }
                }
            }
            app.game_state = Some(game_state);
            continue;
        }

        let p_index = game_state.current_turn;
        let active_id = game_state.players[p_index].id;
        let active_chips = game_state.players[p_index].chips;
        let active_bet = game_state.players[p_index].current_bet;
        let active_hole_cards = game_state.players[p_index].hole_cards.clone();

        if active_id == 0 {
            if event::poll(Duration::from_millis(16)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }
                    match key.code {
                        KeyCode::Esc => { break; }
                        KeyCode::Char(c) => app.handle_char(c),
                        KeyCode::Backspace => app.handle_backspace(),
                        KeyCode::Enter => {
                            let input = app.take_input();
                            let input_trim = input.trim();
                            let amt_trim = input_trim.to_lowercase();
                            let call_amt = game_state.current_highest_bet - active_bet;

                            if app.mode == AppMode::GamePlayRaising {
                                if let Ok(amt) = input_trim.parse::<u32>() {
                                    if let Ok(events) = game_state.process_action(0, PlayerAction::Raise(amt)) {
                                        process_events(&events, &game_state, &mut app.action_log, i18n);
                                    }
                                } else if amt_trim == "all" {
                                    let call_amt = game_state.current_highest_bet - active_bet;
                                    if let Ok(events) = game_state.process_action(0, PlayerAction::Raise(active_chips.saturating_sub(call_amt))) {
                                        process_events(&events, &game_state, &mut app.action_log, i18n);
                                    }
                                }
                                app.mode = AppMode::GamePlay;
                            } else {
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
                                    let raise_amt = std::cmp::min(game_state.min_raise, active_chips.saturating_sub(call_amt));
                                    Some(PlayerAction::Raise(raise_amt))
                                } else {
                                    None
                                };

                                if let Some(action) = action_opt {
                                    if let Ok(events) = game_state.process_action(0, action) {
                                        process_events(&events, &game_state, &mut app.action_log, i18n);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Bot Turn
            let num_opponents = game_state.players.iter().filter(|p| !p.is_folded && p.id != active_id).count();
            let action = compute_bot_action(&game_state, active_id, active_chips, active_bet, &active_hole_cards, num_opponents);
            
            if let Ok(events) = game_state.process_action(active_id, action) {
                process_events(&events, &game_state, &mut app.action_log, i18n);
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        app.game_state = Some(game_state);
    }
}

fn compute_bot_action(
    game: &GameState,
    active_id: usize,
    active_chips: u32,
    active_bet: u32,
    active_hole_cards: &[engine::card::Card],
    num_opponents: usize,
) -> PlayerAction {
    let amount_to_call = game.current_highest_bet - active_bet;
    let is_aggressive = active_id % 2 == 0;

    let win_rate = if num_opponents > 0 {
        engine::ai::calculate_win_rate(
            active_hole_cards,
            &game.community_cards,
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

    let pot_scaled_raise = game.pot / (if is_aggressive { 2 } else { 4 });
    let raise_amount = std::cmp::max(game.min_raise, pot_scaled_raise);
    let total_raise_cost = amount_to_call + raise_amount;

    if win_rate > raise_threshold && active_chips >= total_raise_cost {
        PlayerAction::Raise(raise_amount)
    } else if amount_to_call == 0 {
        PlayerAction::Check
    } else if win_rate > call_threshold {
        PlayerAction::Call
    } else {
        PlayerAction::Fold
    }
}
