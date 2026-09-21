use crate::card::{Card, Rank, Suit};
use rand::rng;
use rand::seq::SliceRandom;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);

        for suit in Suit::all() {
            for rank in Rank::all() {
                cards.push(Card::new(rank, suit));
            }
        }

        let mut deck = Self { cards };
        deck.shuffle();
        deck
    }

    pub fn shuffle(&mut self) {
        let mut rng = rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn remaining_cards(&self) -> usize {
        self.cards.len()
    }
}
