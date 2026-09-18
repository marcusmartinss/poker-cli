use crate::card::Card;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    WaitingForPlayers,
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
    Finished,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PlayerAction {
    Fold,
    Check,
    Call,
    Raise(u32),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum GameEvent {
    PlayerJoined(usize, String),
    GameStarted,
    PhaseChanged(GamePhase),
    CommunityCardsRevealed(Vec<Card>),
    PlayerActed(usize, PlayerAction),
    PotAwarded(usize, u32, String), // id, amount, hand_name
}
