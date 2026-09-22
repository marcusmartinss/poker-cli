use engine::event::GamePhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Portuguese,
}

pub struct I18n {
    pub lang: Language,
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    pub fn t(&self, key: &str) -> &'static str {
        match self.lang {
            Language::English => match key {
                "app_title" => "TERMINAL TEXAS HOLD'EM ♠♥♦♣",
                "new_hand" => "NEW HAND",
                "pot" => "POT",
                "highest_bet" => "HIGHEST BET",
                "board_cards" => "BOARD CARDS:",
                "players" => "PLAYERS:",
                "table_header" => "| {} | Role | Name            | Chips  | Bet     | Status     |",
                "folded" => "(FOLDED)",
                "all_in" => "(ALL-IN)",
                "bet" => "Bet",
                "recent_actions" => "[ RECENT ACTIONS ]",
                "your_turn" => "Your turn!",
                "your_cards" => "Your Hole Cards:",
                "menu_actions" => "Options",
                "menu_fold" => "Fold",
                "menu_check" => "Check",
                "menu_call" => "Call",
                "menu_raise" => "Raise",
                "action_prompt" => "Your move: ",
                "raise_prompt" => "Enter amount to raise ('min' for minimum, 'all' for All-In): ",
                "invalid_amt" => "Invalid amount. Try again.",
                "unknown_cmd" => "Unknown command. Try again.",
                "invalid_move" => "[WARNING] Invalid move:",
                "bot_invalid_move" => "[WARNING] Bot invalid move:",
                "showdown_reveal" => "SHOWDOWN REVEAL",
                "folded_end" => "FOLDED.",
                "cards_end" => "CARDS:",
                "final_actions" => "[ FINAL ACTIONS ]",
                "press_enter" => "Press [Enter] to play the next hand or [Ctrl+C] to exit...",
                "dealer_dealing" => "The dealer is dealing hole cards...",
                "dealer_revealed" => "Dealer revealed:",
                "folded_action" => "folded.",
                "checked_action" => "checked.",
                "called_action" => "called.",
                "raised_by" => "RAISED by",
                "won" => "WON",
                "chips_with" => "CHIPS WITH",
                "chips" => "CHIPS",
                "error_start" => "Error starting game:",
                "phase" => "PHASE",
                "human_name" => "You",
                "bot1_name" => "Conservative Bot",
                "bot2_name" => "Aggressive Bot",
                "Not enough players to start." => "Not enough players to start.",
                "Game is not active." => "Game is not active.",
                "It is not this player's turn." => "It is not this player's turn.",
                "Cannot check. You must call or raise." => "Cannot check. You must call or raise.",
                "Not enough chips to raise that amount." => {
                    "Not enough chips to raise that amount."
                }
                "win_prob" => "Win Probability:",
                "current_hand" => "Current Hand:",
                "you_lost" => "You lost all your chips!",
                "player_busted" => "A player was eliminated!",
                "eliminated" => "(ELIMINATED)",
                "you_won_game" => "You eliminated everyone and WON THE GAME!",
                "select_mode" => "Select Mode:",
                "mode_local" => "Play Local (Offline Bots)",
                "mode_host" => "Host LAN Game",
                "mode_join" => "Join LAN Game",
                _ => "???",
            },
            Language::Portuguese => match key {
                "app_title" => "TEXAS HOLD'EM NO TERMINAL ♠♥♦♣",
                "new_hand" => "NOVA MÃO",
                "pot" => "POTE",
                "highest_bet" => "MAIOR APOSTA",
                "board_cards" => "CARTAS DA MESA:",
                "players" => "JOGADORES:",
                "table_header" => "| {} | Pos. | Nome            | Fichas | Aposta  | Status     |",
                "folded" => "(CORREU)",
                "all_in" => "(TUDO-OU-NADA)",
                "bet" => "Aposta",
                "recent_actions" => "[ ÚLTIMAS AÇÕES ]",
                "your_turn" => "Sua vez!",
                "your_cards" => "Suas Cartas:",
                "menu_actions" => "Opções",
                "menu_fold" => "Correr",
                "menu_check" => "Mesa",
                "menu_call" => "Pagar",
                "menu_raise" => "Aumentar",
                "action_prompt" => "Sua jogada: ",
                "raise_prompt" => "Digite o valor para aumentar ('min' para mínimo, 'all' para All-In): ",
                "invalid_amt" => "Valor inválido. Tente novamente.",
                "unknown_cmd" => "Comando desconhecido. Tente novamente.",
                "invalid_move" => "[AVISO] Movimento inválido:",
                "bot_invalid_move" => "[AVISO] Bot tentou movimento inválido:",
                "showdown_reveal" => "REVELAÇÃO DO SHOWDOWN",
                "folded_end" => "CORREU.",
                "cards_end" => "CARTAS:",
                "final_actions" => "[ AÇÕES FINAIS ]",
                "press_enter" => "Pressione [Enter] para a próxima mão ou [Ctrl+C] para sair...",
                "dealer_dealing" => "O dealer está distribuindo as cartas...",
                "dealer_revealed" => "Dealer revelou:",
                "folded_action" => "correu.",
                "checked_action" => "deu mesa.",
                "called_action" => "pagou a aposta.",
                "raised_by" => "AUMENTOU em",
                "won" => "VENCEU",
                "chips_with" => "FICHAS COM",
                "chips" => "FICHAS",
                "error_start" => "Erro ao iniciar jogo:",
                "phase" => "FASE",
                "human_name" => "Você",
                "bot1_name" => "Bot Conservador",
                "bot2_name" => "Bot Agressivo",
                "Not enough players to start." => "Não há jogadores suficientes para começar.",
                "Game is not active." => "O jogo não está ativo.",
                "It is not this player's turn." => "Não é o turno deste jogador.",
                "Cannot check. You must call or raise." => {
                    "Não pode dar Mesa (Check). Você deve Pagar (Call) ou Aumentar (Raise)."
                }
                "Not enough chips to raise that amount." => {
                    "Fichas insuficientes para aumentar esse valor."
                }
                "win_prob" => "Probabilidade de Vitória:",
                "current_hand" => "Mão Atual:",
                "you_lost" => "Você perdeu todas as suas fichas!",
                "player_busted" => "Um jogador foi eliminado da mesa!",
                "eliminated" => "(ELIMINADO)",
                "you_won_game" => "Você eliminou todos os bots e VENCEU O JOGO!",
                "select_mode" => "Escolha o Modo:",
                "mode_local" => "Jogar Local (Bots Offline)",
                "mode_host" => "Criar Jogo na LAN (Host)",
                "mode_join" => "Entrar em Jogo na LAN",
                _ => "???",
            },
        }
    }

    pub fn t_phase(&self, phase: &GamePhase) -> &'static str {
        if self.lang == Language::English {
            match phase {
                GamePhase::WaitingForPlayers => "Waiting For Players",
                GamePhase::PreFlop => "Pre-Flop",
                GamePhase::Flop => "Flop",
                GamePhase::Turn => "Turn",
                GamePhase::River => "River",
                GamePhase::Showdown => "Showdown",
                GamePhase::Finished => "Finished",
            }
        } else {
            match phase {
                GamePhase::WaitingForPlayers => "Aguardando Jogadores",
                GamePhase::PreFlop => "Pré-Flop",
                GamePhase::Flop => "Flop",
                GamePhase::Turn => "Turn",
                GamePhase::River => "River",
                GamePhase::Showdown => "Showdown",
                GamePhase::Finished => "Finalizado",
            }
        }
    }

    pub fn t_hand(&self, raw: &str) -> String {
        match self.lang {
            Language::English => {
                if raw.contains("Everyone else folded") || raw.contains("everyone else folded") {
                    "Everyone else folded".to_string()
                } else if raw.contains("HighCard") {
                    "High Card".to_string()
                } else if raw.contains("Pair") && !raw.contains("TwoPair") {
                    "One Pair".to_string()
                } else if raw.contains("TwoPair") {
                    "Two Pair".to_string()
                } else if raw.contains("ThreeOfAKind") {
                    "Three of a Kind".to_string()
                } else if raw.contains("Straight") && !raw.contains("Flush") {
                    "Straight".to_string()
                } else if raw.contains("Flush") && !raw.contains("Straight") {
                    "Flush".to_string()
                } else if raw.contains("FullHouse") {
                    "Full House".to_string()
                } else if raw.contains("FourOfAKind") {
                    "Four of a Kind".to_string()
                } else if raw.contains("StraightFlush") {
                    "Straight Flush".to_string()
                } else {
                    raw.to_string()
                }
            }
            Language::Portuguese => {
                if raw.contains("Everyone else folded") || raw.contains("everyone else folded") {
                    "Todos os outros correram".to_string()
                } else if raw.contains("HighCard") {
                    "Carta Alta".to_string()
                } else if raw.contains("Pair") && !raw.contains("TwoPair") {
                    "Um Par".to_string()
                } else if raw.contains("TwoPair") {
                    "Dois Pares".to_string()
                } else if raw.contains("ThreeOfAKind") {
                    "Trinca".to_string()
                } else if raw.contains("Straight") && !raw.contains("Flush") {
                    "Sequência".to_string()
                } else if raw.contains("Flush") && !raw.contains("Straight") {
                    "Flush".to_string()
                } else if raw.contains("FullHouse") {
                    "Full House".to_string()
                } else if raw.contains("FourOfAKind") {
                    "Quadra".to_string()
                } else if raw.contains("StraightFlush") {
                    "Straight Flush".to_string()
                } else {
                    raw.to_string()
                }
            }
        }
    }

    pub fn t_player_cards(&self, is_human: bool, player_name: &str) -> String {
        match self.lang {
            Language::English => {
                if is_human {
                    "Your CARDS:".to_string()
                } else {
                    format!("{}'s CARDS:", player_name)
                }
            }
            Language::Portuguese => {
                if is_human {
                    "Suas CARTAS:".to_string()
                } else {
                    format!("CARTAS de {}:", player_name)
                }
            }
        }
    }
}
