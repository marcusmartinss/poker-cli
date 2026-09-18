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
        let _ = io::stdout().flush();

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!(">> [ERRO CRÍTICO] Falha ao ler a entrada do terminal: {}", e);
            break;
        }
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
                let _ = io::stdout().flush();
                let mut amount_str = String::new();
                if let Err(e) = io::stdin().read_line(&mut amount_str) {
                    println!(">> [ERRO] Falha ao ler a quantidade: {}", e);
                    continue;
                }
                
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
                println!(">> Cartas sendo avaliadas:");
                for c in &table_cards {
                    println!("    - {}", c);
                }
                match evaluate(&table_cards) {
                    Ok(hand_rank) => println!(">> RESULTADO DO MOTOR: {:#?}", hand_rank),
                    Err(err) => println!(">> [ERRO] O motor recusou a avaliação: {}", err),
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
