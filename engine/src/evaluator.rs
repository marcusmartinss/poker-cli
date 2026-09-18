use crate::card::{Card, Rank, Suit};

/// The hierarchy of poker hands, from lowest to highest.
/// Because of the order in the enum and the derived traits, 
/// Rust automatically knows that RoyalFlush > OnePair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    HighCard([Rank; 5]),
    OnePair(Rank, [Rank; 3]),
    TwoPair(Rank, Rank, Rank),
    ThreeOfAKind(Rank, [Rank; 2]),
    Straight(Rank),
    Flush([Rank; 5]),
    FullHouse(Rank, Rank),
    FourOfAKind(Rank, Rank),
    StraightFlush(Rank),
    RoyalFlush,
}

pub fn evaluate(cards: &[Card]) -> HandRank {
    assert!(cards.len() >= 5, "Not enough cards to evaluate");
    // TODO: Implement the full 5-to-7 card evaluation logic
    
    // For now, this is a skeleton that always returns a High Card 
    // just to make it compile and testable.
    let mut sorted = cards.to_vec();
    sorted.sort();
    sorted.reverse(); // Highest first

    HandRank::HighCard([
        sorted[0].rank,
        sorted[1].rank,
        sorted[2].rank,
        sorted[3].rank,
        sorted[4].rank,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_evaluator() {
        let cards = vec![
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Hearts),
        ];

        let result = evaluate(&cards);
        assert_eq!(
            result,
            HandRank::HighCard([Rank::Ace, Rank::King, Rank::Ten, Rank::Five, Rank::Two])
        );
    }
}
