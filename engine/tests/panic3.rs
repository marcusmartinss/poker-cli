use engine::card::{Card, Rank, Suit};
use engine::event::{PlayerAction};
use engine::player::Player;
use engine::state::GameState;

#[test]
fn test_exact_hand_deadlock() {
    let mut game = GameState::new();
    game.players
        .push(Player::new(1000, "Bot Alice".to_string(), 3000)); // D
    game.players
        .push(Player::new(1, "marcus".to_string(), 1000)); // SB
    game.players.push(Player::new(2, "pedro".to_string(), 1000)); // BB

    let _ = game.start_game();

    // Set exact hole cards
    game.players[0].hole_cards = vec![
        Card {
            suit: Suit::Diamonds,
            rank: Rank::Six,
        },
        Card {
            suit: Suit::Spades,
            rank: Rank::King,
        },
    ];
    game.players[1].hole_cards = vec![
        Card {
            suit: Suit::Clubs,
            rank: Rank::Six,
        },
        Card {
            suit: Suit::Spades,
            rank: Rank::Queen,
        },
    ];
    game.players[2].hole_cards = vec![
        Card {
            suit: Suit::Clubs,
            rank: Rank::Two,
        },
        Card {
            suit: Suit::Diamonds,
            rank: Rank::Ten,
        },
    ];

    // Set exact community cards
    game.community_cards = vec![
        Card {
            suit: Suit::Diamonds,
            rank: Rank::Seven,
        },
        Card {
            suit: Suit::Hearts,
            rank: Rank::Six,
        },
        Card {
            suit: Suit::Hearts,
            rank: Rank::Three,
        },
        Card {
            suit: Suit::Clubs,
            rank: Rank::Jack,
        },
        Card {
            suit: Suit::Hearts,
            rank: Rank::Two,
        },
    ];

    game.process_action(1000, PlayerAction::Call).unwrap();
    game.process_action(1, PlayerAction::Call).unwrap();
    game.process_action(2, PlayerAction::Check).unwrap();

    // Flop
    game.process_action(1, PlayerAction::Raise(980)).unwrap();
    game.process_action(2, PlayerAction::Call).unwrap();
    game.process_action(1000, PlayerAction::Call).unwrap();

    // Turn
    game.process_action(1000, PlayerAction::Check).unwrap();

    // River
    let evts = game.process_action(1000, PlayerAction::Check);
    println!("River check result: {:?}", evts);
    println!("Phase after River check: {:?}", game.phase);
}
