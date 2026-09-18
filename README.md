# Texas Hold'em CLI ♠️♥️♣️♦️

Um jogo completo de **Texas Hold'em Poker** desenhado para ser jogado diretamente no seu terminal. Desenvolvido inteiramente em **Rust**, o jogo traz uma interface ASCII limpa, inteligência artificial baseada em simulações de Monte Carlo e suporte a multiplayer via rede local (LAN).

## Funcionalidades

- **Multiplayer LAN (Host & Play):** Hospede um servidor no seu computador e jogue com amigos na mesma rede. O servidor gerencia salas simultâneas de forma robusta utilizando TCP.
- **Descoberta Automática:** Encontra automaticamente servidores rodando na mesma rede Wi-Fi/LAN sem precisar digitar IPs manualmente!
- **Singleplayer com IA:** Jogue offline contra bots controlados pelo computador. Os bots utilizam simulações matemáticas (Monte Carlo) para avaliar a força da mão em tempo real e decidir se devem blefar, apostar, cobrir ou correr.
- **Multilíngue (i18n):** Suporte nativo para **Português (BR)** e **Inglês (EN)**. O idioma pode ser escolhido ao iniciar o jogo.
- **Motor de Regras Preciso:** Validação estrita das regras do Texas Hold'em, incluindo apostas (Raise/Call/Fold), avaliação de mãos (High Card a Royal Flush), All-in e divisão correta de potes.
- **UI Limpa:** Uma interface de terminal renderizada em blocos ASCII, focada na legibilidade, com limpeza de tela inteligente para uma experiência fluida.

## Requisitos

Para rodar o jogo, você precisará ter a linguagem **Rust** e o gerenciador de pacotes **Cargo** instalados no seu sistema.

- [Instale o Rust (rustup)](https://rustup.rs/) (Requer versão 1.70 ou superior).

## Como Rodar

1. Clone o repositório para sua máquina:
   ```bash
   git clone https://github.com/seu-usuario/poker-cli.git
   cd poker-cli
   ```

2. Compile e execute o jogo:
   ```bash
   cargo run --release
   ```
   *Nota: Usar `--release` faz com que os cálculos matemáticos dos bots rodem muito mais rápido.*

## Como Jogar

Ao iniciar o jogo, você poderá escolher o idioma e, em seguida, o modo de jogo:

1. **Play Local (Bots):** Joga uma partida offline contra o "Bot Agressivo" e o "Bot Conservador".
2. **Host LAN Game:** Cria um servidor em background e conecta você automaticamente a um Lobby. Você pode criar salas, adicionar bots dinamicamente e iniciar a partida para quem se conectar.
3. **Join LAN Game:** Busca e lista automaticamente os servidores hospedados na sua rede local. Basta escolher o número correspondente para conectar!

Durante o jogo, basta digitar os números correspondentes às ações na tela (Ex: `1` para Correr, `3` para Pagar, `4` para Aumentar).

## Arquitetura do Projeto

O código-fonte adota as melhores práticas de modularização do Rust, separando a lógica pura do jogo e a interface:

- `engine/` **(Biblioteca):** Motor independente do jogo. Avalia as cartas, gerencia o baralho, aplica as regras do Hold'em, processa os turnos (`state.rs`) e contém o núcleo da Inteligência Artificial (`ai.rs`). Não possui dependências de I/O.
- `cli/` **(Binário):** Aplicação interativa de terminal. Gerencia os menus, a tradução (`i18n.rs`), a renderização ASCII (`ui.rs`), o protocolo de rede serializado via Serde (`net_messages.rs`) e a infraestrutura TCP de Cliente/Servidor com descoberta UDP.

## Contribuindo

Sinta-se livre para abrir *Issues* ou *Pull Requests*. Algumas áreas de melhoria futuras incluem:
- Cálculo avançado de *Side Pots* para múltiplos *All-ins* assimétricos.
- Melhorias estéticas e cores no terminal via crates como `crossterm`.
- Aprimoramento das personalidades dinâmicas da IA na rede.

