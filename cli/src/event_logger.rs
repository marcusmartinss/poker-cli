use crate::i18n::I18n;
use engine::event::{GameEvent, PlayerAction};
use engine::state::GameState;

pub fn process_events(events: &[GameEvent], game: &GameState, log: &mut Vec<String>, i18n: &I18n) {
    for event in events {
        match event {
            GameEvent::GameStarted => {
                log.push(i18n.t("dealer_dealing").to_string());
            }
            GameEvent::PhaseChanged(phase) => {
                log.push(format!(
                    "--- {} {} ---",
                    i18n.t("phase"),
                    i18n.t_phase(phase)
                ));
            }
            GameEvent::CommunityCardsRevealed(cards) => {
                let cards_str = cards
                    .iter()
                    .map(|c| {
                        let is_red = c.suit == engine::card::Suit::Hearts || c.suit == engine::card::Suit::Diamonds;
                        if is_red {
                            format!("{}", c.to_string())
                        } else {
                            c.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                log.push(format!("{} {}", i18n.t("dealer_revealed"), cards_str));
            }
            GameEvent::PlayerActed(id, action) => {
                let name = game
                    .players
                    .iter()
                    .find(|p| p.id == *id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("Unknown");
                match action {
                    PlayerAction::Fold => log.push(format!("{} {}", name, i18n.t("folded_action"))),
                    PlayerAction::Check => {
                        log.push(format!("{} {}", name, i18n.t("checked_action")))
                    }
                    PlayerAction::Call => log.push(format!("{} {}", name, i18n.t("called_action"))),
                    PlayerAction::Raise(amt) => {
                        log.push(format!("{} {} {}", name, i18n.t("raised_by"), amt))
                    }
                }
            }
            GameEvent::PotAwarded(player_id, amount, reason) => {
                let name = game
                    .players
                    .iter()
                    .find(|p| p.id == *player_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("Unknown");

                if reason == "Everyone else folded" {
                    log.push(format!(
                        "*** {} {} {} {}! ***",
                        name,
                        i18n.t("won"),
                        amount,
                        i18n.t("chips")
                    ));
                } else {
                    let hand_translation = i18n.t_hand(reason).to_uppercase();
                    log.push(format!(
                        "*** {} {} {} {} {}! ***",
                        name,
                        i18n.t("won"),
                        amount,
                        i18n.t("chips_with"),
                        hand_translation
                    ));
                }
            }
            _ => {}
        }
    }
}
