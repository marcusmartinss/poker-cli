mod ui;
mod i18n;

use engine::event::{GameEvent, GamePhase, PlayerAction};
use engine::state::GameState;
use std::io::{self, Write};
use i18n::{I18n, Language};

fn get_language() -> Language {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("=========================================================");
    println!("                 TEXAS HOLD'EM CLI                       ");
    println!("=========================================================\n");
    
    println!("  Select Language / Escolha o Idioma:\n");
    println!("    [1] English");
    println!("    [2] Português (Brasil)\n");
    
    print!("  => ");
    let _ = io::stdout().flush();
    
    let mut lang_input = String::new();
    let _ = io::stdin().read_line(&mut lang_input);
    
    if lang_input.trim() == "1" {
        Language::English
    } else {
        Language::Portuguese
    }
}

fn main() {
    let lang = get_language();
    let i18n = I18n::new(lang);

    println!("======================================");
    println!("      {}     ", i18n.t("app_title"));
    println!("======================================");

    let mut game = GameState::new();
    
    game.add_player(0, i18n.t("human_name").to_string(), 1000);
    game.add_player(1, i18n.t("bot1_name").to_string(), 1000);
    game.add_player(2, i18n.t("bot2_name").to_string(), 1000);

    loop {
        println!("\n\n--------------------------------------");
        println!("              {}                ", i18n.t("new_hand"));
        println!("--------------------------------------");

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
            let active_player = &game.players[p_index];
            
            // Calculate number of active opponents for the Monte Carlo simulation
            let num_opponents = game.players.iter().filter(|p| !p.is_folded && p.id != active_player.id).count();

            if active_player.id == 0 {
                ui::render_table(&i18n, &game);
                
                println!("  {}", i18n.t("recent_actions"));
                let recent = if action_log.len() > 5 { &action_log[action_log.len()-5..] } else { &action_log[..] };
                for msg in recent {
                    println!("   > {}", msg);
                }
                println!("---------------------------------------------------------\n");
                
                println!("{}\n", i18n.t("your_turn"));
                
                if active_player.hole_cards.len() == 2 {
                    println!("{}", i18n.t("your_cards"));
                    ui::draw_cards_ascii(&active_player.hole_cards, false);
                    
                    let mut my_cards = game.community_cards.clone();
                    my_cards.extend(active_player.hole_cards.clone());
                    
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

                    if !raw_hand.is_empty() {
                        println!("  {} {}\n", i18n.t("current_hand"), i18n.t_hand(&raw_hand));
                    }
                }

                println!("{}", i18n.t("actions_menu"));
                print!("{}", i18n.t("action_prompt"));
                let _ = io::stdout().flush();
                
                let Some(action) = handle_human_turn(&game, active_player, &i18n, &mut action_log) else {
                    continue;
                };

                match game.process_action(active_player.id, action) {
                    Ok(events) => process_events(&events, &game, &mut action_log, &i18n),
                    Err(e) => action_log.push(format!("{} {}", i18n.t("invalid_move"), i18n.t(e))),
                }
            } else {
                let active_id = active_player.id;
                let action = handle_bot_turn(&game, active_player, num_opponents);
                
                match game.process_action(active_id, action) {
                    Ok(events) => process_events(&events, &game, &mut action_log, &i18n),
                    Err(_) => {
                        let fallback_action = if game.current_highest_bet > game.players.iter().find(|p| p.id == active_id).unwrap().current_bet {
                            PlayerAction::Call
                        } else {
                            PlayerAction::Check
                        };
                        if let Ok(events) = game.process_action(active_id, fallback_action) {
                            process_events(&events, &game, &mut action_log, &i18n);
                        } else {
                            let _ = game.process_action(active_id, PlayerAction::Fold);
                        }
                    }
                }
            }
        }

        ui::render_table(&i18n, &game);
        ui::render_showdown(&i18n, &game);
        
        println!("  {}", i18n.t("final_actions"));
        let recent = if action_log.len() > 8 { &action_log[action_log.len()-8..] } else { &action_log[..] };
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
    active_player: &engine::player::Player,
    i18n: &I18n,
    action_log: &mut Vec<String>,
) -> Option<PlayerAction> {
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    match input.trim() {
        "1" => Some(PlayerAction::Fold),
        "2" => Some(PlayerAction::Check),
        "3" => Some(PlayerAction::Call),
        "4" => {
            print!("{}", i18n.t("raise_prompt"));
            let _ = io::stdout().flush();
            let mut amt_input = String::new();
            let _ = io::stdin().read_line(&mut amt_input);
            
            let amt_trim = amt_input.trim().to_lowercase();
            let call_amt = game.current_highest_bet - active_player.current_bet;
            if amt_trim == "all" {
                if active_player.chips <= call_amt {
                    Some(PlayerAction::Call)
                } else {
                    Some(PlayerAction::Raise(active_player.chips - call_amt))
                }
            } else if let Ok(amt) = amt_trim.parse::<u32>() {
                Some(PlayerAction::Raise(amt))
            } else {
                action_log.push(i18n.t("invalid_amt").to_string());
                None
            }
        },
        _ => {
            action_log.push(i18n.t("unknown_cmd").to_string());
            None
        }
    }
}

fn handle_bot_turn(
    game: &GameState,
    active_player: &engine::player::Player,
    num_opponents: usize,
) -> PlayerAction {
    let amount_to_call = game.current_highest_bet - active_player.current_bet;
    let active_chips = active_player.chips;
    let is_aggressive = active_player.name.contains("Agressivo") || active_player.name.contains("Aggressive");
    
    let win_rate = if num_opponents > 0 {
        engine::ai::calculate_win_rate(
            &active_player.hole_cards,
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

    let raise_amount = if is_aggressive { 150 } else { 50 };
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
    if let Some(human) = game.players.iter().find(|p| p.id == 0) {
        if human.chips == 0 {
            println!("\n  >>> GAME OVER! {} <<<", i18n.t("you_lost"));
            return true;
        }
    } else {
        println!("\n  >>> GAME OVER! {} <<<", i18n.t("you_lost"));
        return true;
    }

    let old_count = game.players.len();
    game.players.retain(|p| p.chips > 0);
    let new_count = game.players.len();

    if old_count > new_count {
        println!("  >>> {} <<<", i18n.t("player_busted"));
    }

    if game.players.len() == 1 {
        println!("\n  >>> PARABÉNS! {} <<<", i18n.t("you_won_game"));
        return true;
    }

    println!("\n{}", i18n.t("press_enter"));
    let _ = io::stdout().flush();
    let mut _input = String::new();
    let _ = io::stdin().read_line(&mut _input);

    game.dealer_button = (game.dealer_button + 1) % game.players.len();
    false
}

fn process_events(events: &[GameEvent], game: &GameState, log: &mut Vec<String>, i18n: &I18n) {
    for event in events {
        match event {
            GameEvent::GameStarted => {
                log.push(i18n.t("dealer_dealing").to_string());
            }
            GameEvent::PhaseChanged(phase) => {
                log.push(format!("--- {} {} ---", i18n.t("phase"), i18n.t_phase(phase)));
            }
            GameEvent::CommunityCardsRevealed(cards) => {
                let cards_str = cards.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
                log.push(format!("{} {}", i18n.t("dealer_revealed"), cards_str));
            }
            GameEvent::PlayerActed(id, action) => {
                let name = game.players.iter().find(|p| p.id == *id).map(|p| p.name.as_str()).unwrap_or("Unknown");
                match action {
                    PlayerAction::Fold => log.push(format!("{} {}", name, i18n.t("folded_action"))),
                    PlayerAction::Check => log.push(format!("{} {}", name, i18n.t("checked_action"))),
                    PlayerAction::Call => log.push(format!("{} {}", name, i18n.t("called_action"))),
                    PlayerAction::Raise(amt) => log.push(format!("{} {} {}!", name, i18n.t("raised_by"), amt)),
                }
            }
            GameEvent::PotAwarded(id, amount, hand_desc) => {
                let name = game.players.iter().find(|p| p.id == *id).map(|p| p.name.as_str()).unwrap_or("Unknown");
                let local_hand = i18n.t_hand(hand_desc);
                log.push(format!("*** {} {} {} {} {}! ***", name, i18n.t("won"), amount, i18n.t("chips_with"), local_hand.to_uppercase()));
            }
            _ => {}
        }
    }
}
