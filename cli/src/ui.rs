use engine::card::Card;
use engine::state::GameState;

pub fn draw_cards_ascii(cards: &[Card], is_board: bool) {
    if cards.is_empty() && !is_board {
        return;
    }

    let mut lines = vec![String::new(); 5];
    let total_cards = if is_board { 5 } else { cards.len() };

    for i in 0..total_cards {
        if i < cards.len() {
            let r = cards[i].rank.to_string();
            let s = cards[i].suit.to_string();
            
            let pad_left = if r.len() == 2 { "" } else { " " };
            let pad_right = if r.len() == 2 { "" } else { " " };
            
            lines[0].push_str("┌───────┐ ");
            lines[1].push_str(&format!("│ {}{}    │ ", r, pad_left));
            lines[2].push_str(&format!("│   {}   │ ", s));
            lines[3].push_str(&format!("│    {}{} │ ", pad_right, r));
            lines[4].push_str("└───────┘ ");
        } else if is_board {
            // Draw empty placeholders for community cards not yet dealt
            lines[0].push_str("┌───────┐ ");
            lines[1].push_str("│ ░░░░░ │ ");
            lines[2].push_str("│ ░░░░░ │ ");
            lines[3].push_str("│ ░░░░░ │ ");
            lines[4].push_str("└───────┘ ");
        }
    }

    for line in lines {
        println!("    {}", line);
    }
}

pub fn render_table(i18n: &crate::i18n::I18n, game: &GameState) {
    // Clear screen and reset cursor
    print!("{}[2J{}[1;1H", 27 as char, 27 as char);

    println!("=========================================================");
    println!("  {}: {}  |  {}: ${}  |  {}: ${}", 
        i18n.t("phase"), i18n.t_phase(&game.phase), 
        i18n.t("pot"), game.pot, 
        i18n.t("highest_bet"), game.current_highest_bet);
    println!("=========================================================\n");

    println!("  {}", i18n.t("board_cards"));
    draw_cards_ascii(&game.community_cards, true);
    println!();

    println!("---------------------------------------------------------");
    println!("  {}", i18n.t("players"));
    let num_players = game.players.len();
    for (i, p) in game.players.iter().enumerate() {
        let is_dealer = game.dealer_button == i;
        let is_sb = (game.dealer_button + 1) % num_players == i;
        let is_bb = (game.dealer_button + 2) % num_players == i;

        let role_token = if is_dealer {
            "[D]"
        } else if is_sb {
            "[SB]"
        } else if is_bb {
            "[BB]"
        } else {
            ""
        };

        let active_token = if game.current_turn == i { "=>" } else { "  " };
        
        let status = if p.is_folded {
            format!("({})", i18n.t("folded"))
        } else if p.is_all_in {
            format!("({})", i18n.t("all_in"))
        } else {
            "".to_string()
        };

        // Truncate name
        let mut name = p.name.clone();
        if name.chars().count() > 15 {
            name = name.chars().take(12).collect::<String>();
            name.push_str("...");
        }

        println!("  {} {:<4} {:<15} | ${:<6} | {}: ${:<6} {}", 
            active_token, role_token, name, p.chips, i18n.t("bet"), p.current_bet, status);
    }
    println!("---------------------------------------------------------\n");
}

pub fn render_showdown(i18n: &crate::i18n::I18n, game: &GameState) {
    println!("\n=========================================================");
    println!("                 {}                         ", i18n.t("showdown_reveal"));
    println!("=========================================================");
    for p in &game.players {
        if p.is_folded {
            println!("\n  {} {}", p.name, i18n.t("folded_end"));
        } else {
            // Evaluate their hand for display
            let mut all_cards = game.community_cards.clone();
            all_cards.extend(p.hole_cards.clone());
            let raw_hand = if all_cards.len() == 2 {
                if all_cards[0].rank == all_cards[1].rank { "Pair".to_string() } else { "HighCard".to_string() }
            } else {
                match engine::evaluator::evaluate(&all_cards) {
                    Ok(rank) => format!("{:?}", rank),
                    Err(_) => "Unknown".to_string(),
                }
            };
            let hand_name = i18n.t_hand(&raw_hand);
            
            let possessive_str = i18n.t_player_cards(p.id == 0, &p.name);
            println!("\n  {} - {} ", possessive_str, hand_name);
            draw_cards_ascii(&p.hole_cards, false);
        }
    }
    println!("=========================================================\n");
}
