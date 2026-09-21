use engine::event::{GamePhase, PlayerAction};
use engine::player::Player;
use engine::state::GameState;

#[test]
fn test_all_in_panic() {
    let mut game = GameState::new();
    game.players
        .push(Player::new(1000, "Bot Alice".to_string(), 1000));
    game.players
        .push(Player::new(1, "marcus".to_string(), 1000));
    game.players.push(Player::new(2, "paulo".to_string(), 1000));

    let _ = game.start_game();

    // Everyone goes all in pre-flop
    game.process_action(1000, PlayerAction::Raise(980)).unwrap(); // Bot Alice goes all in (1000 total)
    game.process_action(1, PlayerAction::Call).unwrap(); // marcus all in
    game.process_action(2, PlayerAction::Call).unwrap(); // paulo all in

    // Phase should INSTANTLY be Finished!
    println!("Phase is now: {:?}", game.phase);
    assert_eq!(game.phase, GamePhase::Finished);
}
