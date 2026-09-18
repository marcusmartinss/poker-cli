use crate::card::Card;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub chips: u32,
    pub hole_cards: Vec<Card>,
    pub current_bet: u32,
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
            is_folded: false,
            is_all_in: false,
            has_acted: false,
        }
    }
}
