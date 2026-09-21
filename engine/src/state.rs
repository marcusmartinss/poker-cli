use crate::card::Card;
use crate::deck::Deck;
use crate::event::{GameEvent, GamePhase, PlayerAction};
use crate::player::Player;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GameState {
    pub phase: GamePhase,
    pub players: Vec<Player>,
    pub deck: Deck,
    pub community_cards: Vec<Card>,
    pub pot: u32,
    pub current_turn: usize,
    pub dealer_button: usize,
    pub current_highest_bet: u32,
    pub min_raise: u32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::WaitingForPlayers,
            players: Vec::new(),
            deck: Deck::new(),
            community_cards: Vec::with_capacity(5),
            pot: 0,
            current_turn: 0,
            dealer_button: 0,
            current_highest_bet: 0,
            min_raise: 20,
        }
    }

    pub fn add_player(&mut self, id: usize, name: String, chips: u32) {
        self.players.push(Player::new(id, name, chips));
    }

    pub fn start_game(&mut self) -> Result<Vec<GameEvent>, &'static str> {
        if self.players.len() < 2 {
            return Err("Not enough players to start.");
        }

        self.phase = GamePhase::PreFlop;
        self.deck = Deck::new();
        self.community_cards.clear();
        self.pot = 0;
        self.current_highest_bet = 0;
        self.min_raise = 20; // Default minimum raise is big blind

        for player in &mut self.players {
            player.is_folded = false;
            player.is_all_in = false;
            player.has_acted = false;
            player.current_bet = 0;
            player.hole_cards.clear();
            if let Some(c1) = self.deck.draw() {
                player.hole_cards.push(c1);
            }
            if let Some(c2) = self.deck.draw() {
                player.hole_cards.push(c2);
            }
        }

        // Apply Blinds
        let sb_index = (self.dealer_button + 1) % self.players.len();
        let bb_index = (self.dealer_button + 2) % self.players.len();

        self.apply_forced_bet(sb_index, 10);
        self.apply_forced_bet(bb_index, 20);
        self.current_highest_bet = 20;
        self.min_raise = 20;

        // Turn starts with the player after the Big Blind (UTG)
        self.current_turn = (self.dealer_button + 3) % self.players.len();

        Ok(vec![
            GameEvent::GameStarted,
            GameEvent::PhaseChanged(self.phase),
        ])
    }

    pub fn process_action(
        &mut self,
        player_id: usize,
        action: PlayerAction,
    ) -> Result<Vec<GameEvent>, &'static str> {
        if self.phase == GamePhase::WaitingForPlayers || self.phase == GamePhase::Finished {
            return Err("Game is not active.");
        }

        let p_index = self.current_turn;
        if self.players[p_index].id != player_id {
            return Err("It is not this player's turn.");
        }

        let mut events = Vec::new();

        self.apply_action(p_index, &action)?;
        events.push(GameEvent::PlayerActed(player_id, action));

        if self.active_players_count() == 1 {
            self.handle_early_win(&mut events);
            return Ok(events);
        }

        if self.is_betting_round_complete() {
            self.advance_phase(&mut events);
        } else {
            self.advance_turn();
        }

        Ok(events)
    }

    fn apply_forced_bet(&mut self, player_index: usize, amount: u32) {
        let p = &mut self.players[player_index];
        let actual = amount.min(p.chips);
        p.chips -= actual;
        p.current_bet += actual;
        self.pot += actual;
        if p.chips == 0 {
            p.is_all_in = true;
        }
    }

    fn apply_action(
        &mut self,
        player_index: usize,
        action: &PlayerAction,
    ) -> Result<(), &'static str> {
        let player = &mut self.players[player_index];
        player.has_acted = true;

        match action {
            PlayerAction::Fold => {
                player.is_folded = true;
            }
            PlayerAction::Check => {
                if player.current_bet < self.current_highest_bet {
                    return Err("Cannot check. You must call or raise.");
                }
            }
            PlayerAction::Call => {
                let amount_to_call = self.current_highest_bet - player.current_bet;
                let actual_call = amount_to_call.min(player.chips);

                player.chips -= actual_call;
                player.current_bet += actual_call;
                self.pot += actual_call;

                if player.chips == 0 {
                    player.is_all_in = true;
                }
            }
            PlayerAction::Raise(amount) => {
                let total_needed = (self.current_highest_bet - player.current_bet) + *amount;
                let is_all_in_raise = player.chips == total_needed;
                
                if !is_all_in_raise && *amount < self.min_raise {
                    return Err("Raise amount is below the minimum allowed raise.");
                }
                
                if player.chips < total_needed {
                    return Err("Not enough chips to raise that amount.");
                }

                player.chips -= total_needed;
                player.current_bet += total_needed;
                self.pot += total_needed;
                self.current_highest_bet = player.current_bet;
                if *amount > self.min_raise {
                    self.min_raise = *amount;
                }

                if player.chips == 0 {
                    player.is_all_in = true;
                }

                for (i, p) in self.players.iter_mut().enumerate() {
                    if i != player_index && !p.is_folded && !p.is_all_in {
                        p.has_acted = false;
                    }
                }
            }
        }
        Ok(())
    }

    fn is_betting_round_complete(&self) -> bool {
        self.players.iter().all(|p| {
            p.is_folded || p.is_all_in || (p.has_acted && p.current_bet == self.current_highest_bet)
        })
    }

    fn advance_turn(&mut self) {
        for _ in 0..self.players.len() {
            self.current_turn = (self.current_turn + 1) % self.players.len();
            let p = &self.players[self.current_turn];
            if !p.is_folded && !p.is_all_in {
                break;
            }
        }
    }

    fn advance_phase(&mut self, events: &mut Vec<GameEvent>) {
        for p in &mut self.players {
            p.current_bet = 0;
            p.has_acted = false;
        }
        self.current_highest_bet = 0;
        self.min_raise = 20; // Default minimum raise is big blind

        match self.phase {
            GamePhase::PreFlop => {
                self.phase = GamePhase::Flop;
                self.deal_community_cards(3, events);
            }
            GamePhase::Flop => {
                self.phase = GamePhase::Turn;
                self.deal_community_cards(1, events);
            }
            GamePhase::Turn => {
                self.phase = GamePhase::River;
                self.deal_community_cards(1, events);
            }
            GamePhase::River => {
                self.handle_showdown(events);
                return;
            }
            _ => {}
        }

        // After dealing cards, the first active player left of the dealer acts
        self.current_turn = self.dealer_button;
        self.advance_turn();

        // If everyone is all-in (or folded), the betting round is instantly complete.
        // We should recursively advance the phase until Showdown/Finished.
        if self.phase != GamePhase::Finished && self.is_betting_round_complete() {
            self.advance_phase(events);
        }
    }

    fn handle_showdown(&mut self, events: &mut Vec<GameEvent>) {
        self.phase = GamePhase::Showdown;
        events.push(GameEvent::PhaseChanged(self.phase));

        let mut best_rank: Option<crate::evaluator::HandRank> = None;
        let mut winners: Vec<usize> = Vec::new();

        for (i, p) in self.players.iter().enumerate() {
            if p.is_folded {
                continue;
            }

            let mut all_cards = self.community_cards.clone();
            all_cards.extend(p.hole_cards.clone());

            if let Ok(rank) = crate::evaluator::evaluate(&all_cards) {
                match best_rank {
                    None => {
                        best_rank = Some(rank);
                        winners.push(i);
                    }
                    Some(ref best) => {
                        if rank > *best {
                            best_rank = Some(rank);
                            winners.clear();
                            winners.push(i);
                        } else if rank == *best {
                            winners.push(i);
                        }
                    }
                }
            }
        }

        if !winners.is_empty() {
            let split_amount = self.pot / winners.len() as u32;
            let hand_desc = match &best_rank {
                Some(r) => format!("{:?}", r),
                None => "Unknown".to_string(),
            };

            for &idx in &winners {
                self.players[idx].chips += split_amount;
                events.push(GameEvent::PotAwarded(
                    self.players[idx].id,
                    split_amount,
                    hand_desc.clone(),
                ));
            }
        }

        self.phase = GamePhase::Finished;
        events.push(GameEvent::PhaseChanged(self.phase));
    }

    fn deal_community_cards(&mut self, count: usize, events: &mut Vec<GameEvent>) {
        events.push(GameEvent::PhaseChanged(self.phase));
        let mut drawn = Vec::new();
        for _ in 0..count {
            if let Some(card) = self.deck.draw() {
                self.community_cards.push(card);
                drawn.push(card);
            }
        }
        events.push(GameEvent::CommunityCardsRevealed(drawn));
    }

    fn active_players_count(&self) -> usize {
        self.players.iter().filter(|p| !p.is_folded).count()
    }

    fn handle_early_win(&mut self, events: &mut Vec<GameEvent>) {
        self.phase = GamePhase::Finished;
        events.push(GameEvent::PhaseChanged(self.phase));

        if let Some(winner) = self.players.iter_mut().find(|p| !p.is_folded) {
            winner.chips += self.pot;
            events.push(GameEvent::PotAwarded(
                winner.id,
                self.pot,
                "Everyone else folded".to_string(),
            ));
        }
    }
}
