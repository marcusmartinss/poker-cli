use engine::event::{GamePhase, PlayerAction};
use engine::player::Player;
use engine::state::GameState;

#[test]
fn test_panic5() {
    let mut game = GameState::new();
    game.players
        .push(Player::new(1000, "Bot Alice".to_string(), 1000));
    game.players
        .push(Player::new(1, "marcus".to_string(), 1000));
    game.players.push(Player::new(2, "pedro".to_string(), 1000));

    // Give them cards
    game.players[0].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];
    game.players[1].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];
    game.players[2].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];

    game.phase = GamePhase::PreFlop;
    game.pot = 30; // 10 SB + 20 BB
    game.current_highest_bet = 20;

    // Set up bets
    game.players[1].current_bet = 10;
    game.players[1].chips = 990;
    game.players[2].current_bet = 20;
    game.players[2].chips = 980;

    game.current_turn = 0; // Bot Alice
    game.process_action(1000, PlayerAction::Call).unwrap(); // Calls 20

    // Now Marcus raises
    game.process_action(1, PlayerAction::Raise(980)).unwrap();
    // Pedro calls
    game.process_action(2, PlayerAction::Call).unwrap();

    // Now it's Bot Alice again
    let evts = game.process_action(1000, PlayerAction::Call).unwrap();
    println!("Bot Alice call: {:?}", evts);
}

#[test]
fn test_serde_massive() {
    let mut game = GameState::new();
    game.players
        .push(Player::new(1000, "Bot Alice".to_string(), 1000));
    game.players
        .push(Player::new(1, "marcus".to_string(), 1000));
    game.players.push(Player::new(2, "pedro".to_string(), 1000));

    game.players[0].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];
    game.players[1].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];
    game.players[2].hole_cards = vec![game.deck.draw().unwrap(), game.deck.draw().unwrap()];

    game.phase = GamePhase::PreFlop;
    game.pot = 30;
    game.current_highest_bet = 20;

    game.players[1].current_bet = 10;
    game.players[1].chips = 990;
    game.players[2].current_bet = 20;
    game.players[2].chips = 980;

    game.current_turn = 0;
    game.process_action(1000, PlayerAction::Call).unwrap();
    game.process_action(1, PlayerAction::Raise(980)).unwrap();
    game.process_action(2, PlayerAction::Call).unwrap();

    let evts = game.process_action(1000, PlayerAction::Call).unwrap();

    // Simulate what the server does
    #[derive(serde::Serialize)]
    enum ServerMessage {
        GameUpdate {
            state: GameState,
            events: Vec<engine::event::GameEvent>,
            your_id: usize,
        },
    }
    let msg = ServerMessage::GameUpdate {
        state: game,
        events: evts,
        your_id: 1,
    };
    let json = serde_json::to_string(&msg).unwrap();
    println!("Serialized length: {}", json.len());
}
