use engine::card::{Card, Rank, Suit};
use engine::event::{GameEvent, GamePhase, PlayerAction};
use engine::state::GameState;

#[test]
fn test_serde() {
    let game = GameState::new();
    let events = vec![
        GameEvent::PlayerActed(1000, PlayerAction::Call),
        GameEvent::PhaseChanged(GamePhase::Flop),
        GameEvent::CommunityCardsRevealed(vec![Card::new(Rank::King, Suit::Hearts)]),
        GameEvent::PhaseChanged(GamePhase::Turn),
        GameEvent::CommunityCardsRevealed(vec![Card::new(Rank::King, Suit::Hearts)]),
        GameEvent::PhaseChanged(GamePhase::River),
        GameEvent::CommunityCardsRevealed(vec![Card::new(Rank::King, Suit::Hearts)]),
        GameEvent::PhaseChanged(GamePhase::Showdown),
        GameEvent::PotAwarded(1000, 3000, "FullHouse".to_string()),
        GameEvent::PhaseChanged(GamePhase::Finished),
    ];
    let json_game = serde_json::to_string(&game).unwrap();
    let json_events = serde_json::to_string(&events).unwrap();
    println!("Serialized length game: {}", json_game.len());
    println!("Serialized length events: {}", json_events.len());
}
