use crate::card::{Card, Rank};
use itertools::Itertools;
use std::collections::HashMap;

/// The hierarchy of poker hands, from lowest to highest.
/// The order matters: `#[derive(Ord)]` tells Rust that RoyalFlush > StraightFlush, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    HighCard([Rank; 5]),
    OnePair(Rank, [Rank; 3]),
    TwoPair(Rank, Rank, Rank),     // High pair, Low pair, Kicker
    ThreeOfAKind(Rank, [Rank; 2]), // Triple, 2 Kickers
    Straight(Rank),                // Highest card in the straight
    Flush([Rank; 5]),
    FullHouse(Rank, Rank),         // Triple, Pair
    FourOfAKind(Rank, Rank),       // Quad, Kicker
    StraightFlush(Rank),           // Highest card
    RoyalFlush,
}

/// Evaluates 5 to 7 cards and returns the absolute best hand rank possible.
pub fn evaluate(cards: &[Card]) -> Result<HandRank, &'static str> {
    if cards.len() < 5 || cards.len() > 7 {
        return Err("Evaluator requires exactly 5 to 7 cards.");
    }

    // Generate all 5-card combinations. 
    // E.g., for 7 cards (2 hole + 5 community), this generates 21 combinations.
    let combinations = cards.iter().copied().combinations(5);

    // Evaluate each combination and return the maximum (best) hand.
    let best_hand = combinations
        .map(|combo| {
            let mut arr = [combo[0], combo[1], combo[2], combo[3], combo[4]];
            evaluate_5_cards(&mut arr)
        })
        .max()
        .ok_or("Erro interno do motor: Não foi possível gerar combinações de 5 cartas.")?;

    Ok(best_hand)
}

/// Evaluates exactly 5 cards.
fn evaluate_5_cards(cards: &mut [Card; 5]) -> HandRank {
    // 1. Sort cards by rank in descending order
    cards.sort_by(|a, b| b.rank.cmp(&a.rank));
    
    // 2. Check for Flush (all suits match the first card's suit)
    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);

    // 3. Check for Straight
    let mut is_straight = true;
    for i in 0..4 {
        if cards[i].rank as i32 - 1 != cards[i+1].rank as i32 {
            is_straight = false;
            break;
        }
    }
    
    // Special Poker Rule: Low straight (A-5-4-3-2)
    let is_low_straight = cards[0].rank == Rank::Ace 
                       && cards[1].rank == Rank::Five 
                       && cards[2].rank == Rank::Four 
                       && cards[3].rank == Rank::Three 
                       && cards[4].rank == Rank::Two;
                       
    if is_low_straight {
        is_straight = true;
        // In A-5-4-3-2, the highest straight card is 5, not Ace.
        // We rotate the array to put the 5 at the front: [5, 4, 3, 2, A]
        cards.rotate_left(1); 
    }

    let highest_straight_card = cards[0].rank;

    // 4. Check for Straight Flush and Royal Flush
    if is_flush && is_straight {
        if highest_straight_card == Rank::Ace && !is_low_straight {
            return HandRank::RoyalFlush;
        }
        return HandRank::StraightFlush(highest_straight_card);
    }
    
    if is_flush {
        return HandRank::Flush([cards[0].rank, cards[1].rank, cards[2].rank, cards[3].rank, cards[4].rank]);
    }
    
    if is_straight {
        return HandRank::Straight(highest_straight_card);
    }

    // 5. Count rank frequencies for Pairs, Triples, and Quads
    let mut counts: HashMap<Rank, u8> = HashMap::new();
    for c in cards.iter() {
        *counts.entry(c.rank).or_insert(0) += 1;
    }

    // Convert to a vector of (Frequency, Rank) and sort descending
    // E.g., for a Full House (K, K, K, 2, 2) it becomes [(3, King), (2, Two)]
    let mut freq: Vec<(u8, Rank)> = counts.into_iter().map(|(r, c)| (c, r)).collect();
    freq.sort_by(|a, b| b.cmp(a));

    // Analyze the frequency table
    match freq[0].0 {
        4 => {
            // Four of a kind
            HandRank::FourOfAKind(freq[0].1, freq[1].1)
        },
        3 => {
            if freq[1].0 == 2 {
                // Full House
                HandRank::FullHouse(freq[0].1, freq[1].1)
            } else {
                // Three of a kind + 2 Kickers
                HandRank::ThreeOfAKind(freq[0].1, [freq[1].1, freq[2].1])
            }
        },
        2 => {
            if freq[1].0 == 2 {
                // Two Pair + Kicker
                HandRank::TwoPair(freq[0].1, freq[1].1, freq[2].1)
            } else {
                // One Pair + 3 Kickers
                HandRank::OnePair(freq[0].1, [freq[1].1, freq[2].1, freq[3].1])
            }
        },
        _ => {
            // High Card
            HandRank::HighCard([cards[0].rank, cards[1].rank, cards[2].rank, cards[3].rank, cards[4].rank])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Suit;

    #[test]
    fn test_royal_flush_with_7_cards() {
        let cards = vec![
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::Queen, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Ten, Suit::Hearts),
            Card::new(Rank::Two, Suit::Spades),  // Garbage hole card
            Card::new(Rank::Three, Suit::Clubs), // Garbage hole card
        ];
        assert_eq!(evaluate(&cards).unwrap(), HandRank::RoyalFlush);
    }
    
    #[test]
    fn test_full_house_beats_flush() {
        let full_house = vec![
            Card::new(Rank::Ten, Suit::Hearts),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Ten, Suit::Clubs),
            Card::new(Rank::Four, Suit::Hearts),
            Card::new(Rank::Four, Suit::Diamonds),
        ];
        
        let flush = vec![
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Nine, Suit::Spades),
            Card::new(Rank::Eight, Suit::Spades),
            Card::new(Rank::Two, Suit::Spades),
        ];
        
        // Full house should be strictly greater than flush
        assert!(evaluate(&full_house).unwrap() > evaluate(&flush).unwrap());
    }
}
