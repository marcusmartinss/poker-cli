use engine::state::GameState;
use engine::player::Player;
use engine::event::{PlayerAction, GamePhase};
use engine::ai::calculate_win_rate;
use std::time::Instant;

#[test]
fn test_simulate_full_game() {
    let start = Instant::now();
    let mut game = GameState::new();
    
    // Add 4 bots
    game.players.push(Player::new(1, "Bot Alice".to_string(), 1000));
    game.players.push(Player::new(2, "Bot Bob".to_string(), 1000));
    game.players.push(Player::new(3, "Bot Charlie".to_string(), 1000));
    game.players.push(Player::new(4, "Bot Dave".to_string(), 1000));

    let _ = game.start_game();
    
    let mut turns_played = 0;
    while game.phase != GamePhase::Finished && turns_played < 100 {
        let p_index = game.current_turn;
        let active_id = game.players[p_index].id;
        let active_chips = game.players[p_index].chips;
        let active_bet = game.players[p_index].current_bet;
        let active_hole_cards = game.players[p_index].hole_cards.clone();
        
        let num_opponents = game.players.iter().filter(|p| !p.is_folded && p.id != active_id).count();
        let amount_to_call = game.current_highest_bet - active_bet;
        let is_aggressive = active_id % 2 == 0;

        let action = if num_opponents > 0 {
            let win_rate = calculate_win_rate(
                &active_hole_cards,
                &game.community_cards,
                num_opponents,
                1000
            );
            
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
            } else if win_rate > call_threshold && active_chips >= amount_to_call {
                PlayerAction::Call
            } else {
                PlayerAction::Fold
            }
        } else {
            PlayerAction::Fold
        };

        let _ = game.process_action(active_id, action.clone());
        
        // Failsafe for invalid bot actions
        if game.current_turn == p_index && game.phase != GamePhase::Finished {
            let fallback = if amount_to_call > 0 { PlayerAction::Call } else { PlayerAction::Check };
            if game.process_action(active_id, fallback).is_err() {
                let _ = game.process_action(active_id, PlayerAction::Fold);
            }
        }
        turns_played += 1;
    }
    
    let duration = start.elapsed();
    println!("Simulated a full game of {} turns in {:?}", turns_played, duration);
    
    // Test passes if it successfully completes a hand within a reasonable time
    assert!(game.phase == GamePhase::Finished);
    assert!(duration.as_millis() < 5000, "Simulation took too long: {:?}", duration);
}
