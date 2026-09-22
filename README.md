# Texas Hold'em CLI ♠️♥️♣️♦️

Um jogo completo de **Texas Hold'em Poker** desenhado para ser jogado diretamente no seu terminal. Desenvolvido inteiramente em **Rust**, o jogo foca em uma interface ASCII rápida, multiplayer descentralizado (LAN) e uma IA inteligente.

## 🚀 Como Começar

### 1. Instale o Rust (Cargo)
Se você usa Linux ou macOS, instale o Rust com o comando oficial:
```bash
curl https://sh.rustup.rs -sSf | sh
```
*(Para Windows, acesse [rustup.rs](https://rustup.rs/))*

### 2. Baixe e Rode o Jogo
```bash
git clone https://github.com/seu-usuario/poker-cli.git
cd poker-cli
cargo run
```
*Dica:* Para que os cálculos da IA offline rodem quase instantaneamente, use `cargo run --release`.

## 🎮 Como Jogar

O jogo suporta **Inglês** e **Português**. No menu principal, você escolhe seu modo:

- **Play Local:** Treine contra bots offline (Agressivo e Conservador).
- **Host LAN Game:** Cria um servidor no seu PC e te coloca num Lobby. Seus amigos na mesma rede Wi-Fi podem entrar na sua sala facilmente.
- **Join LAN Game:** Escaneia sua rede local automaticamente buscando por Hosts (sem necessidade de digitar IP manualmente) e lista os servidores disponíveis.

### Comandos da Mesa
- Responda aos menus numéricos (`1` Correr, `2` Mesa/Pagar, `3` Aumentar).
- Atalhos de aposta: Ao aumentar, digite `min` para a aposta mínima permitida, ou `all` para dar All-In.
- **Chat:** Digite `/c sua mensagem` em qualquer momento para enviar mensagens aos outros jogadores no painel de "Últimas Ações".

## 🛠️ Destaques da Engine

- **Economia Deep Stack & Torneio:** Todos iniciam com impressionantes **$10.000**. Os *Blinds* começam em `$100/$200` e sobem dinamicamente a cada 5 mãos, forçando a ação.
- **Muck Rule:** Blefou e todo mundo correu? O jogo garante o mistério não revelando as suas cartas!
- **Monte Carlo AI:** A inteligência artificial calcula a probabilidade de vitória simulando milhares de desfechos futuros com as cartas da mesa antes de agir.
- **P2P Robusto:** O cliente possui resiliência contra falhas de rede. Se a conexão cair, você é redirecionado suavemente ao menu principal sem que o jogo crashe no terminal.

## 📦 Arquitetura
O projeto usa uma separação limpa:
- `engine/`: O núcleo lógico. Independente, gerenciando estado, baralho, validando regras de aposta e avaliando a força da mão.
- `cli/`: A interface do usuário e rede TCP/UDP. Lida com renderização ASCII, internacionalização (i18n), serialização e servidores assíncronos.
