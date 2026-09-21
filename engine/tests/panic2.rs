use engine::event::{GamePhase, PlayerAction};
use engine::player::Player;
use engine::state::GameState;

#[test]
fn test_bot_not_all_in_deadlock() {
    let mut game = GameState::new();
    game.players
        .push(Player::new(1000, "Bot Alice".to_string(), 3000));
    game.players
        .push(Player::new(1, "marcus".to_string(), 1000));
    game.players.push(Player::new(2, "pedro".to_string(), 1000));

    let _ = game.start_game();

    // Pre-flop: Everyone calls (10, 20)
    game.process_action(1000, PlayerAction::Call).unwrap(); // Bot Alice calls 20
    game.process_action(1, PlayerAction::Call).unwrap(); // marcus calls 20
    game.process_action(2, PlayerAction::Check).unwrap(); // pedro checks BB

    // Flop
    game.process_action(1, PlayerAction::Raise(980)).unwrap(); // marcus goes all-in (980)
    game.process_action(2, PlayerAction::Call).unwrap(); // pedro goes all-in
    game.process_action(1000, PlayerAction::Call).unwrap(); // Bot Alice calls (she is NOT all-in!)

    println!("Phase after Flop call: {:?}", game.phase);

    // Turn
    game.process_action(1000, PlayerAction::Check).unwrap();
    println!("Phase after Turn check: {:?}", game.phase);

    // River
    let evts = game.process_action(1000, PlayerAction::Check);
    println!("River check result: {:?}", evts);
    println!("Phase after River check: {:?}", game.phase);
}
