use std::io::{self, Write};

mod client;
mod event_logger;
mod i18n;
mod local;
mod net_messages;
mod server;
mod ui;
mod tui;

pub use event_logger::process_events;
use i18n::{I18n, Language};

fn main() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("=========================================================");
    println!("                 TEXAS HOLD'EM CLI                       ");
    println!("=========================================================\n");

    println!("  Select Language / Escolha o Idioma:\n");
    println!("    [1] English");
    println!("    [2] Português (Brasil)\n");
    print!("  => ");
    let _ = io::stdout().flush();

    let mut lang_input = String::new();
    let _ = io::stdin().read_line(&mut lang_input);

    let lang = match lang_input.trim() {
        "1" => Language::English,
        "2" => Language::Portuguese,
        _ => Language::English,
    };

    let i18n = I18n::new(lang);

    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("=========================================================");
    println!("                 TEXAS HOLD'EM CLI                       ");
    println!("=========================================================\n");
    println!("  {}", i18n.t("select_mode"));
    println!("    [1] {}", i18n.t("mode_local"));
    println!("    [2] {}", i18n.t("mode_host"));
    println!("    [3] {}", i18n.t("mode_join"));

    loop {
        print!("\n  => ");
        let _ = io::stdout().flush();

        let mut mode_input = String::new();
        let _ = io::stdin().read_line(&mut mode_input);

        match mode_input.trim() {
            "2" => {
                // Start server in background
                std::thread::spawn(|| {
                    server::start_server(8080);
                });
                // Give server a moment to start
                std::thread::sleep(std::time::Duration::from_millis(500));
                // Connect to our own server
                client::start_client("127.0.0.1", 8080, &i18n);
            }
            "3" => {
                println!("\n  Searching for servers on the local network (UDP broadcast)...");
                let mut servers = Vec::new();
                if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
                    if socket.set_broadcast(true).is_ok() {
                        if socket
                            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                            .is_ok()
                        {
                            let _ = socket.send_to(b"POKER_DISCOVER", "255.255.255.255:8081");

                            let mut buf = [0; 64];
                            while let Ok((amt, src)) = socket.recv_from(&mut buf) {
                                if amt >= 12 && &buf[0..12] == b"POKER_SERVER" {
                                    let ip = src.ip().to_string();
                                    let mut name = if amt > 13 { String::from_utf8_lossy(&buf[13..amt]).to_string() } else { "Unknown Server".to_string() };
                                    if name.is_empty() {
                                        name = "Unknown Server".to_string();
                                    }
                                    if !servers.iter().any(|(s_ip, _)| s_ip == &ip) {
                                        servers.push((ip, name));
                                    }
                                }
                            }
                        }
                    }
                }

                let final_ip = if servers.is_empty() {
                    println!("  No servers found.");
                    print!("  Enter IP Address manually (default 127.0.0.1): ");
                    let _ = io::stdout().flush();
                    let mut ip_input = String::new();
                    let _ = io::stdin().read_line(&mut ip_input);
                    let mut ip = ip_input.trim().to_string();
                    if ip.is_empty() {
                        ip = "127.0.0.1".to_string();
                    }
                    ip
                } else {
                    println!("\n  Found {} server(s):", servers.len());
                    for (i, (_ip, name)) in servers.iter().enumerate() {
                        println!("    [{}] {}", i + 1, name);
                    }
                    println!("    [M] Enter IP Manually");
                    print!("  => ");
                    let _ = io::stdout().flush();

                    let mut sel_input = String::new();
                    let _ = io::stdin().read_line(&mut sel_input);
                    let sel = sel_input.trim();

                    if let Ok(idx) = sel.parse::<usize>() {
                        if idx > 0 && idx <= servers.len() {
                            servers[idx - 1].0.clone()
                        } else {
                            "127.0.0.1".to_string()
                        }
                    } else {
                        print!("  Enter IP Address: ");
                        let _ = io::stdout().flush();
                        let mut ip_input = String::new();
                        let _ = io::stdin().read_line(&mut ip_input);
                        let mut ip = ip_input.trim().to_string();
                        if ip.is_empty() {
                            ip = "127.0.0.1".to_string();
                        }
                        ip
                    }
                };

                client::start_client(&final_ip, 8080, &i18n);
            }
            _ => {
                local::play_local(&i18n);
            }
        }
        
        // Repaint menu after a game session ends or connection fails
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("=========================================================");
        println!("                 TEXAS HOLD'EM CLI                       ");
        println!("=========================================================\n");
        println!("  {}", i18n.t("select_mode"));
        println!("    [1] {}", i18n.t("mode_local"));
        println!("    [2] {}", i18n.t("mode_host"));
        println!("    [3] {}", i18n.t("mode_join"));
    }
}
