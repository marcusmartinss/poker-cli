use crate::card::Card;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub chips: u32,
    pub hole_cards: Vec<Card>,
    pub current_bet: u32,
    pub invested_in_hand: u32,
    pub is_folded: bool,
    pub is_all_in: bool,
    pub has_acted: bool,
}

impl Player {
    pub fn new(id: usize, name: String, chips: u32) -> Self {
        Self {
            id,
            name,
            chips,
            hole_cards: Vec::with_capacity(2),
            current_bet: 0,
            invested_in_hand: 0,
            is_folded: false,
            is_all_in: false,
            has_acted: false,
        }
    }
}
