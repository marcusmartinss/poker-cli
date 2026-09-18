use engine::deck::Deck;
use engine::evaluator::evaluate;
use std::io::{self, Write};

fn main() {
    println!("♠♥♦♣ TESTE ♠♥♦♣");

    let mut deck = Deck::new();
    let mut table_cards = Vec::new();

    loop {
        println!("\n==================================");
        println!("ESTADO ATUAL:");
        println!("- Cartas restantes no baralho: {}", deck.remaining_cards());
        println!("- Cartas na mesa (sua mão): {}", table_cards.len());
        println!("==================================");
        
        println!("1. Criar novo baralho de 52 cartas (Reset)");
        println!("2. Embaralhar o baralho atual");
        println!("3. Sacar 1 carta do topo");
        println!("4. Sacar X cartas do topo");
        println!("5. Avaliar as cartas que estão na mesa");
        println!("6. Limpar a mesa (jogar cartas fora)");
        println!("0. Sair");
        print!("\nEscolha uma opção: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                deck = Deck::new();
                table_cards.clear();
                println!(">> Novo baralho de 52 cartas gerado na ordem original.");
            }
            "2" => {
                deck.shuffle();
                println!(">> Baralho embaralhado criptograficamente!");
            }
            "3" => {
                match deck.draw() {
                    Some(card) => {
                        println!(">> Você sacou: {}", card);
                        table_cards.push(card);
                    }
                    None => {
                        println!(">> [ALERTA] O baralho acabou! Não é possível sacar mais cartas.");
                    }
                }
            }
            "4" => {
                print!("Quantas cartas quer sacar? ");
                io::stdout().flush().unwrap();
                let mut amount_str = String::new();
                io::stdin().read_line(&mut amount_str).unwrap();
                
                if let Ok(amount) = amount_str.trim().parse::<usize>() {
                    let mut drawn = 0;
                    for _ in 0..amount {
                        if let Some(card) = deck.draw() {
                            table_cards.push(card);
                            drawn += 1;
                        } else {
                            println!(">> [ALERTA] O baralho secou no meio do caminho! Apenas {} cartas foram sacadas.", drawn);
                            break;
                        }
                    }
                    println!(">> {} cartas adicionadas à mesa.", drawn);
                } else {
                    println!(">> [ERRO] Número inválido.");
                }
            }
            "5" => {
                if table_cards.len() < 5 {
                    println!(">> [ERRO] Erro do Avaliador: Você precisa de no mínimo 5 cartas na mesa para avaliar. Tem apenas {}.", table_cards.len());
                } else {
                    println!(">> Cartas sendo avaliadas:");
                    for c in &table_cards {
                        println!("    - {}", c);
                    }
                    let hand_rank = evaluate(&table_cards);
                    println!(">> RESULTADO DO MOTOR: {:#?}", hand_rank);
                }
            }
            "6" => {
                table_cards.clear();
                println!(">> A mesa foi limpa. (As cartas foram descartadas, elas NÃO voltam pro baralho)");
            }
            "0" => {
                println!(">> Encerrando o laboratório. Até logo!");
                break;
            }
            _ => {
                println!(">> [ERRO] Opção inválida.");
            }
        }
    }
}
