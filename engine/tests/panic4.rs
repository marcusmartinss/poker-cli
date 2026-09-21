use engine::event::{GamePhase, PlayerAction};
use engine::player::Player;
use engine::state::GameState;

#[test]
fn test_panic4() {
    let mut game = GameState::new();
    game.players
        .push(Player::new(1000, "Bot Alice".to_string(), 3000));
    game.players.push(Player::new(1, "marcus".to_string(), 990));
    game.players.push(Player::new(2, "pedro".to_string(), 980));

    // Give them cards so evaluation works
    game.players[0].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];
    game.players[1].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];
    game.players[2].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];

    // Pre-flop setup
    game.phase = GamePhase::Flop;
    game.pot = 60;
    game.current_highest_bet = 0;
    game.current_turn = 1; // Marcus acts first on Flop

    game.process_action(1, PlayerAction::Raise(980)).unwrap();
    game.process_action(2, PlayerAction::Call).unwrap();
    game.process_action(1000, PlayerAction::Call).unwrap();

    println!(
        "Current turn after Flop: {}, ID: {}",
        game.current_turn, game.players[game.current_turn].id
    );
}
