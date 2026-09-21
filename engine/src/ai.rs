use crate::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use rayon::prelude::*;

pub fn calculate_win_rate(
    my_cards: &[Card],
    community_cards: &[Card],
    num_opponents: usize,
    iterations: usize,
) -> f64 {
    if my_cards.len() != 2 || num_opponents == 0 {
        return 0.0;
    }

    // Build standard 52 deck, removing known cards
    let mut full_deck = Vec::with_capacity(52);
    let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
    let ranks = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    for s in &suits {
        for r in &ranks {
            let c = Card::new(*r, *s);
            // Check if card is in my_cards or community_cards
            let mut is_known = false;
            for known in my_cards {
                if known.suit == *s && known.rank == *r {
                    is_known = true;
                    break;
                }
            }
            if !is_known {
                for known in community_cards {
                    if known.suit == *s && known.rank == *r {
                        is_known = true;
                        break;
                    }
                }
            }

            if !is_known {
                full_deck.push(c);
            }
        }
    }

    let (wins, ties) = (0..iterations)
        .into_par_iter()
        .map_init(
            || rand::rng(),
            |rng, _| {
                let mut sim_deck = full_deck.clone();
                sim_deck.shuffle(rng);

                // Deal remaining community cards
                let needed_community = 5 - community_cards.len();
                let mut sim_community = community_cards.to_vec();
                for _ in 0..needed_community {
                    sim_community.push(sim_deck.pop().unwrap());
                }

                // Evaluate my hand
                let mut my_full = sim_community.clone();
                my_full.extend_from_slice(my_cards);

                let my_rank = match crate::evaluator::evaluate(&my_full) {
                    Ok(r) => r,
                    Err(_) => return (0, 0),
                };

                // Deal opponents and evaluate
                let mut won = true;
                let mut tied = false;

                for _ in 0..num_opponents {
                    let c1 = sim_deck.pop().unwrap();
                    let c2 = sim_deck.pop().unwrap();
                    let mut opp_full = sim_community.clone();
                    opp_full.push(c1);
                    opp_full.push(c2);

                    if let Ok(opp_rank) = crate::evaluator::evaluate(&opp_full) {
                        if opp_rank > my_rank {
                            won = false;
                            tied = false;
                            break;
                        } else if opp_rank == my_rank {
                            tied = true;
                        }
                    }
                }

                if won {
                    if tied { (0, 1) } else { (1, 0) }
                } else {
                    (0, 0)
                }
            },
        )
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));

    // Ties can count as half a win for Expected Value
    (wins as f64 + (ties as f64 / 2.0)) / iterations as f64
}
