use crate::event_logger::process_events;
use crate::i18n::I18n;
use crate::ui;
use engine::event::{GamePhase, PlayerAction};
use engine::player::Player;
use engine::state::GameState;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

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

    game.players
        .push(Player::new(0, i18n.t("human_name").to_string(), 1000));
    game.players
        .push(Player::new(1, "Bot Alice".to_string(), 1000));
    game.players
        .push(Player::new(2, "Bot Bob".to_string(), 1000));
    game.players
        .push(Player::new(bot3_id, "Bot Charlie".to_string(), 1000));

    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("\n\n--------------------------------------");
        println!("              {}                ", i18n.t("new_hand"));
        println!("--------------------------------------");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::thread::sleep(std::time::Duration::from_millis(2500));

        let mut action_log: Vec<String> = Vec::new();

        match game.start_game() {
            Ok(events) => process_events(&events, &game, &mut action_log, &i18n),
            Err(e) => {
                println!("{} {}", i18n.t("error_start"), e);
                break;
            }
        }

        while game.phase != GamePhase::Finished {
            let p_index = game.current_turn;

            // Extract properties to avoid holding borrow during mutable operations
            let active_id = game.players[p_index].id;
            let active_chips = game.players[p_index].chips;
            let active_bet = game.players[p_index].current_bet;
            let active_name = game.players[p_index].name.clone();
            let active_hole_cards = game.players[p_index].hole_cards.clone();

            let num_opponents = game
                .players
                .iter()
                .filter(|p| !p.is_folded && p.id != active_id)
                .count();

            if active_id == 0 {
                ui::render_table(&i18n, &game);

                println!("  {}", i18n.t("recent_actions"));
                let recent = if action_log.len() > 5 {
                    &action_log[action_log.len() - 5..]
                } else {
                    &action_log[..]
                };
                for msg in recent {
                    println!("   > {}", msg);
                }
                println!("---------------------------------------------------------\n");

                println!("{}\n", i18n.t("your_turn"));

                if active_hole_cards.len() == 2 {
                    let mut my_cards = game.community_cards.clone();
                    my_cards.extend(active_hole_cards.clone());

                    let raw_hand = if my_cards.len() == 2 {
                        if my_cards[0].rank == my_cards[1].rank {
                            "Pair".to_string()
                        } else {
                            "HighCard".to_string()
                        }
                    } else {
                        match engine::evaluator::evaluate(&my_cards) {
                            Ok(rank) => format!("{:?}", rank),
                            Err(_) => "".to_string(),
                        }
                    };

                    println!("{} ({})", i18n.t("your_cards"), i18n.t_hand(&raw_hand));
                    ui::draw_cards_ascii(&active_hole_cards, false);
                }

                if let Some(action) = handle_human_turn(&game, active_chips, active_bet, &i18n) {
                    match game.process_action(active_id, action) {
                        Ok(events) => process_events(&events, &game, &mut action_log, &i18n),
                        Err(e) => {
                            action_log.push(format!("{} {}", i18n.t("invalid_move"), i18n.t(e)))
                        }
                    }
                }
            } else {
                let action = handle_bot_turn(
                    &game,
                    active_id,
                    active_chips,
                    active_bet,
                    &active_name,
                    &active_hole_cards,
                    num_opponents,
                );
                match game.process_action(active_id, action.clone()) {
                    Ok(events) => process_events(&events, &game, &mut action_log, &i18n),
                    Err(_) => {
                        let fallback = if (game.current_highest_bet - active_bet) > 0 {
                            PlayerAction::Call
                        } else {
                            PlayerAction::Check
                        };
                        if let Ok(events) = game.process_action(active_id, fallback) {
                            process_events(&events, &game, &mut action_log, &i18n);
                        } else {
                            let _ = game.process_action(active_id, PlayerAction::Fold);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(150));
            }
        }

        ui::render_table(&i18n, &game);
        let active_count = game.players.iter().filter(|p| !p.is_folded).count();
        if active_count > 1 {
            ui::render_showdown(&i18n, &game);
        }

        println!("  {}", i18n.t("final_actions"));
        let recent = if action_log.len() > 8 {
            &action_log[action_log.len() - 8..]
        } else {
            &action_log[..]
        };
        for msg in recent {
            println!("   > {}", msg);
        }
        println!("---------------------------------------------------------\n");

        if handle_end_of_hand(&mut game, &i18n) {
            break;
        }
    }
}

fn handle_human_turn(
    game: &GameState,
    active_chips: u32,
    active_bet: u32,
    i18n: &I18n,
) -> Option<PlayerAction> {
    let call_amt = game.current_highest_bet - active_bet;
    let menu_str = if call_amt == 0 {
        format!("{}: [1] {} | [2] {} | [3] {}", i18n.t("menu_actions"), i18n.t("menu_fold"), i18n.t("menu_check"), i18n.t("menu_raise"))
    } else {
        format!("{}: [1] {} | [2] {} (${}) | [3] {}", i18n.t("menu_actions"), i18n.t("menu_fold"), i18n.t("menu_call"), call_amt, i18n.t("menu_raise"))
    };
    println!("{}", menu_str);
    print!("{}", i18n.t("action_prompt"));
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let input = input.trim();

    match input {
        "1" => Some(PlayerAction::Fold),
        "2" => if call_amt == 0 { Some(PlayerAction::Check) } else { Some(PlayerAction::Call) },
        "3" => {
            print!("{} (Min: {}): ", i18n.t("raise_prompt").trim_end_matches(": "), game.min_raise);
            let _ = io::stdout().flush();
            let mut amt_input = String::new();
            let _ = io::stdin().read_line(&mut amt_input);
            let amt_trim = amt_input.trim().to_lowercase();

            let call_amt = game.current_highest_bet - active_bet;

            if amt_trim == "all" {
                if active_chips <= call_amt {
                    Some(PlayerAction::Call)
                } else {
                    Some(PlayerAction::Raise(active_chips - call_amt))
                }
            } else if amt_trim == "min" {
                let available_to_raise = active_chips.saturating_sub(call_amt);
                if available_to_raise == 0 {
                    Some(PlayerAction::Call)
                } else {
                    let raise_amt = std::cmp::min(game.min_raise, available_to_raise);
                    Some(PlayerAction::Raise(raise_amt))
                }
            } else if let Ok(amt) = amt_trim.parse::<u32>() {
                Some(PlayerAction::Raise(amt))
            } else {
                println!("{}", i18n.t("invalid_amount"));
                None
            }
        }
        _ => {
            println!("{}", i18n.t("unknown_cmd"));
            None
        }
    }
}

fn handle_bot_turn(
    game: &GameState,
    active_id: usize,
    active_chips: u32,
    active_bet: u32,
    _active_name: &str,
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
            2000,
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

    let raise_amount = std::cmp::max(game.min_raise, if is_aggressive { 150 } else { 50 });
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

fn handle_end_of_hand(game: &mut GameState, i18n: &I18n) -> bool {
    
    if game.players.len() == 1 {
        println!("\n  >>> PARABÉNS! {} <<<", i18n.t("you_won_game"));
        return true;
    }

    println!("\n{}", i18n.t("press_enter"));
    let _ = io::stdout().flush();
    let mut _input = String::new();
    let _ = io::stdin().read_line(&mut _input);

    game.dealer_button = game.next_active_player(game.dealer_button);
    false
}
