use engine::event::{GameEvent, PlayerAction};
use engine::state::GameState;
use crate::i18n::I18n;

pub fn process_events(events: &[GameEvent], game: &GameState, log: &mut Vec<String>, i18n: &I18n) {
    for event in events {
        match event {
            GameEvent::GameStarted => {
                log.push(i18n.t("dealer_dealing").to_string());
            }
            GameEvent::PhaseChanged(phase) => {
                log.push(format!("--- {} {} ---", i18n.t("phase"), i18n.t_phase(phase)));
            }
            GameEvent::CommunityCardsRevealed(cards) => {
                let cards_str = cards.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
                log.push(format!("{} {}", i18n.t("dealer_revealed"), cards_str));
            }
            GameEvent::PlayerActed(id, action) => {
                let name = game.players.iter().find(|p| p.id == *id).map(|p| p.name.as_str()).unwrap_or("Unknown");
                match action {
                    PlayerAction::Fold => log.push(format!("{} {}", name, i18n.t("folded"))),
                    PlayerAction::Check => log.push(format!("{} {}", name, i18n.t("checked"))),
                    PlayerAction::Call => log.push(format!("{} {}", name, i18n.t("called"))),
                    PlayerAction::Raise(amt) => log.push(format!("{} {} {}", name, i18n.t("raised"), amt)),
                }
            }
            GameEvent::PotAwarded(player_id, amount, _) => {
                let name = game.players.iter().find(|p| p.id == *player_id).map(|p| p.name.as_str()).unwrap_or("Unknown");
                
                // Get player cards for the log
                let local_hand = if let Some(p) = game.players.iter().find(|p| p.id == *player_id) {
                    if p.hole_cards.is_empty() {
                        "Unknown".to_string()
                    } else {
                        let mut my_cards = game.community_cards.clone();
                        my_cards.extend(p.hole_cards.clone());
                        let raw_hand = if my_cards.len() == 2 {
                            if my_cards[0].rank == my_cards[1].rank { "Pair".to_string() } else { "HighCard".to_string() }
                        } else {
                            match engine::evaluator::evaluate(&my_cards) {
                                Ok(rank) => format!("{:?}", rank),
                                Err(_) => "Unknown".to_string(),
                            }
                        };
                        i18n.t_hand(&raw_hand).to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                log.push(format!("*** {} {} {} {} {}! ***", name, i18n.t("won"), amount, i18n.t("chips_with"), local_hand.to_uppercase()));
            }
            _ => {}
        }
    }
}
