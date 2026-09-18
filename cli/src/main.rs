mod ui;
mod i18n;

use engine::event::{GameEvent, GamePhase, PlayerAction};
use engine::state::GameState;
use std::io::{self, Write};
use i18n::{I18n, Language};

fn get_language() -> Language {
    print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    println!("Welcome to Terminal Texas Hold'em!");
    println!("Select Language / Selecione o Idioma:");
    println!("[1] English");
    println!("[2] Português");
    print!("> ");
    let _ = io::stdout().flush();
    
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    
    if input.trim() == "2" {
        Language::Portuguese
    } else {
        Language::English
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

            if active_player.id == 0 {
                ui::render_table(&i18n, &game);
                
                println!("  {}", i18n.t("recent_actions"));
                let recent = if action_log.len() > 5 { &action_log[action_log.len()-5..] } else { &action_log[..] };
                for msg in recent {
                    println!("   > {}", msg);
                }
                println!("---------------------------------------------------------\n");
                
                println!("{}, {}!", i18n.t("your_turn"), active_player.name);
                
                if active_player.hole_cards.len() == 2 {
                    println!("{}", i18n.t("your_cards"));
                    ui::draw_cards_ascii(&active_player.hole_cards, false);
                }

                println!("\n{}", i18n.t("actions_menu"));
                print!("{}", i18n.t("action_prompt"));
                let _ = io::stdout().flush();
                
                let mut input = String::new();
                if let Err(_) = io::stdin().read_line(&mut input) {
                    break;
                }

                let action = match input.trim() {
                    "1" => PlayerAction::Fold,
                    "2" => PlayerAction::Check,
                    "3" => PlayerAction::Call,
                    "4" => PlayerAction::Raise(50),
                    _ => {
                        action_log.push(i18n.t("unknown_cmd").to_string());
                        if game.current_highest_bet > active_player.current_bet {
                            PlayerAction::Fold
                        } else {
                            PlayerAction::Check
                        }
                    }
                };

                match game.process_action(active_player.id, action) {
                    Ok(events) => process_events(&events, &game, &mut action_log, &i18n),
                    Err(e) => action_log.push(format!("{} {}", i18n.t("invalid_move"), i18n.t(e))),
                }
            } else {
                let amount_to_call = game.current_highest_bet - active_player.current_bet;
                let action = if amount_to_call == 0 {
                    PlayerAction::Check
                } else if amount_to_call > 0 && amount_to_call <= active_player.chips {
                    PlayerAction::Call
                } else {
                    PlayerAction::Fold
                };

                match game.process_action(active_player.id, action) {
                    Ok(events) => process_events(&events, &game, &mut action_log, &i18n),
                    Err(e) => action_log.push(format!("{} {}", i18n.t("bot_invalid_move"), i18n.t(e))),
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

        println!("\n{}", i18n.t("press_enter"));
        let _ = io::stdout().flush();
        let mut _input = String::new();
        let _ = io::stdin().read_line(&mut _input);

        game.dealer_button = (game.dealer_button + 1) % game.players.len();
    }
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
                let name = &game.players[*id].name;
                match action {
                    PlayerAction::Fold => log.push(format!("{} {}", name, i18n.t("folded_action"))),
                    PlayerAction::Check => log.push(format!("{} {}", name, i18n.t("checked_action"))),
                    PlayerAction::Call => log.push(format!("{} {}", name, i18n.t("called_action"))),
                    PlayerAction::Raise(amt) => log.push(format!("{} {} {}!", name, i18n.t("raised_by"), amt)),
                }
            }
            GameEvent::PotAwarded(id, amount, hand_desc) => {
                let name = &game.players[*id].name;
                let local_hand = i18n.t_hand(hand_desc);
                log.push(format!("🏆 {} {} {} {} {}! 🏆", name, i18n.t("won"), amount, i18n.t("chips_with"), local_hand.to_uppercase()));
            }
            _ => {}
        }
    }
}
