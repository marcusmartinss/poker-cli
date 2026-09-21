use std::io::{self, Write};

mod client;
mod event_logger;
mod i18n;
mod local;
mod net_messages;
mod server;
mod ui;

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
    println!("    [1] Play Local (Offline Bots)");
    println!("    [2] Host LAN Game");
    println!("    [3] Join LAN Game\n");
    print!("  => ");
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

                        let mut buf = [0; 32];
                        while let Ok((amt, src)) = socket.recv_from(&mut buf) {
                            if &buf[..amt] == b"POKER_SERVER" {
                                let ip = src.ip().to_string();
                                if !servers.contains(&ip) {
                                    servers.push(ip);
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
                for (i, ip) in servers.iter().enumerate() {
                    println!("    [{}] {}", i + 1, ip);
                }
                println!("    [M] Enter IP Manually");
                print!("  => ");
                let _ = io::stdout().flush();

                let mut sel_input = String::new();
                let _ = io::stdin().read_line(&mut sel_input);
                let sel = sel_input.trim();

                if let Ok(idx) = sel.parse::<usize>() {
                    if idx > 0 && idx <= servers.len() {
                        servers[idx - 1].clone()
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
}
