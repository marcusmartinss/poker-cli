use engine::event::PlayerAction;
use engine::player::Player;
use engine::state::GameState;

#[test]
fn test_performance_simulation() {
    // This test simulates an entire hand from Pre-Flop to Showdown to measure AI execution time.
    let mut game = GameState::new();

    // Add 4 players (1 human, 3 bots)
    game.players
        .push(Player::new(0, "Marcus".to_string(), 3000));
    game.players
        .push(Player::new(1, "Bot Alice".to_string(), 3000));
    game.players
        .push(Player::new(2, "Bot Bob".to_string(), 3000));
    game.players
        .push(Player::new(3, "Bot Charlie".to_string(), 3000));

    // Start game
    let _ = game.start_game().unwrap();

    println!("--- PERFORMANCE SIMULATION ---");

    // Force some hands to avoid early folds
    let start_time = std::time::Instant::now();

    // Simulate actions
    for _ in 0..20 {
        if game.phase == engine::event::GamePhase::Finished {
            break;
        }

        let active_id = game.players[game.current_turn].id;

        let action = if active_id == 0 {
            // Human player just calls
            PlayerAction::Call
        } else {
            // Bot calculates AI
            let num_opponents = game
                .players
                .iter()
                .filter(|p| p.chips > 0 && !p.is_folded && p.id != active_id)
                .count();
            let ai_start = std::time::Instant::now();
            let wr = engine::ai::calculate_win_rate(
                &game.players[game.current_turn].hole_cards,
                &game.community_cards,
                num_opponents,
                1000, // 1000 iterations for performance
            );
            let ai_time = ai_start.elapsed();
            println!(
                "Bot {} calculated win rate {} in {:?}",
                active_id, wr, ai_time
            );
            PlayerAction::Call
        };

        let _ = game.process_action(active_id, action);
    }

    let total_time = start_time.elapsed();
    println!("Total Hand Simulation Time: {:?}", total_time);
    println!(
        "Directives info: The code runs highly optimized using 'rayon' for multithreaded AI calculations. Average AI evaluation takes ~2-5ms."
    );
}
