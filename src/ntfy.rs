use curl::easy::Easy;
use notify_rust::{Notification, Urgency};
use rich_rust::Console;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Debug)]
struct NtfyMessage {
    #[serde(default)]
    event: String,
    time: i64,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    message: String,
    #[serde(default)]
    priority: Option<u8>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    icon: Option<String>,
}

fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn icon_cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache").join("rustfy").join("icons")
}

fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

fn download_icon(url: &str, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.follow_location(true)?;
    easy.timeout(Duration::from_secs(30))?;

    let mut data = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|chunk| {
            data.extend_from_slice(chunk);
            Ok(chunk.len())
        })?;
        transfer.perform()?;
    }

    if easy.response_code()? != 200 {
        return Err(format!("HTTP {}", easy.response_code()?).into());
    }

    fs::create_dir_all(dest.parent().unwrap())?;
    fs::write(dest, &data)?;
    Ok(())
}

fn get_local_icon_path(icon_url: &str) -> Option<String> {
    let trimmed = icon_url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Some(trimmed.to_string());
    }

    let cache_dir = icon_cache_dir();
    let hash = hash_url(trimmed);
    let cached_file = cache_dir.join(&hash);

    if cached_file.exists() {
        if let Ok(metadata) = fs::metadata(&cached_file) {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now().duration_since(modified).unwrap_or_default();
                if age.as_secs() < 172_800 {
                    return Some(cached_file.to_string_lossy().to_string());
                }
            }
        }
    }

    if let Err(e) = download_icon(trimmed, &cached_file) {
        eprintln!("Aviso: falha ao baixar ícone '{}': {}", trimmed, e);
        return None;
    }

    Some(cached_file.to_string_lossy().to_string())
}

fn clean_icon_cache() {
    let cache_dir = icon_cache_dir();
    if !cache_dir.exists() {
        return;
    }

    let entries = match fs::read_dir(&cache_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let age = SystemTime::now().duration_since(modified).unwrap_or_default();
                    if age.as_secs() >= 172_800 {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

fn send_notification(title: &str, message: &str, priority: Option<u8>, tags: &[String], icon: Option<&str>) {
    let urgency = match priority {
        Some(1) | Some(2) => Urgency::Low,
        Some(4) => Urgency::Normal,
        Some(5) => Urgency::Critical,
        _ => Urgency::Normal,
    };

    let mut display_title = title.to_string();
    if !tags.is_empty() {
        let emoji_tags: Vec<&str> = tags
            .iter()
            .filter(|t| t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            .map(|t| t.as_str())
            .collect();
        if !emoji_tags.is_empty() {
            display_title = format!("[{}] {}", emoji_tags.join(", "), display_title);
        }
    }

    let mut notif = Notification::new();
    notif.appname("rustfy")
        .summary(&display_title)
        .body(message)
        .urgency(urgency);

    if let Some(icon_value) = icon {
        if let Some(local_path) = get_local_icon_path(icon_value) {
            notif.icon(&local_path);
        }
    }

    let _ = notif.show();
}

pub fn listen_topic(base_url: String, topic: String, reconnect_delay: u64, connected_count: Arc<AtomicUsize>) {
    clean_icon_cache();

    let console = Arc::new(Console::new());
    let last_time: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));

    loop {
        let since = {
            let t = last_time.lock().unwrap();
            if *t > 0 {
                t.to_string()
            } else {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                now.to_string()
            }
        };

        let url = format!(
            "{}/{}/json?since={}",
            base_url.trim_end_matches('/'),
            topic,
            since
        );

        let mut easy = Easy::new();
        easy.url(&url).expect("URL inválida");
        easy.follow_location(true).unwrap();
        easy.timeout(Duration::from_secs(60)).unwrap();

        let console_clone = Arc::clone(&console);
        let topic_clone = topic.clone();
        let last_time_clone = Arc::clone(&last_time);
        let mut buffer = Vec::new();
        let has_received_data = Arc::new(Mutex::new(false));
        let has_received_data_clone = Arc::clone(&has_received_data);

        easy.write_function(move |data| {
            {
                let mut received = has_received_data_clone.lock().unwrap();
                *received = true;
            }
            buffer.extend_from_slice(data);

            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.drain(..=pos).collect::<Vec<u8>>();
                let line_str = String::from_utf8_lossy(&line).trim().to_string();

                if line_str.is_empty() {
                    continue;
                }

                match serde_json::from_str::<NtfyMessage>(&line_str) {
                    Ok(msg) if msg.event == "message" => {
                        {
                            let mut t = last_time_clone.lock().unwrap();
                            *t = msg.time + 1;
                        }
                        let title = msg.title.as_deref().unwrap_or(&topic_clone);
                        let timestamp = now_iso();
                        let extras = [
                            msg.priority.map(|p| format!("priority={}", p)),
                            if msg.tags.is_empty() { None } else { Some(format!("tags={:?}", msg.tags)) },
                            msg.icon.as_ref().map(|i| format!("icon={}", i)),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(", ");

                        if extras.is_empty() {
                            console_clone.print(&format!(
                                "[dim]{}[/] [cyan]ℹ[/] Nova mensagem em \"{}\": \"{}\"",
                                timestamp, topic_clone, msg.message
                            ));
                        } else {
                            console_clone.print(&format!(
                                "[dim]{}[/] [cyan]ℹ[/] Nova mensagem em \"{}\" ({}): \"{}\"",
                                timestamp, topic_clone, extras, msg.message
                            ));
                        }
                        send_notification(title, &msg.message, msg.priority, &msg.tags, msg.icon.as_deref());
                    }
                    Ok(_) => {}
                    Err(e) => {
                        let timestamp = now_iso();
                        console_clone.print(&format!(
                            "[dim]{}[/] [red]✗[/] Erro ao parsear mensagem: {}",
                            timestamp, e
                        ));
                    }
                }
            }

            Ok(data.len())
        })
        .unwrap();

        let timestamp = now_iso();
        console.print(&format!(
            "[dim]{}[/] [green]✓[/] Conectado ao tópico: \"{}\"",
            timestamp, topic
        ));

        // Marca como conectado
        connected_count.fetch_add(1, Ordering::Relaxed);

        match easy.perform() {
            Ok(()) => {
                // Desconectou — decrementa antes de qualquer decisão
                connected_count.fetch_sub(1, Ordering::Relaxed);

                match easy.response_code() {
                    Ok(code) if code >= 200 && code < 300 => {
                        let timestamp = now_iso();
                        console.print(&format!(
                            "[dim]{}[/] [dim]↻ Reconectando ao tópico \"{}\"...[/]",
                            timestamp, topic
                        ));
                    }
                    Ok(code) => {
                        let timestamp = now_iso();
                        console.print(&format!(
                            "[dim]{}[/] [yellow]⚠[/] HTTP {} no tópico \"{}\", reconectando em {}s...",
                            timestamp, code, topic, reconnect_delay
                        ));
                        thread::sleep(Duration::from_secs(reconnect_delay));
                    }
                    Err(e) => {
                        let timestamp = now_iso();
                        console.print(&format!(
                            "[dim]{}[/] [yellow]⚠[/] Erro no tópico \"{}\": {}, reconectando em {}s...",
                            timestamp, topic, e, reconnect_delay
                        ));
                        thread::sleep(Duration::from_secs(reconnect_delay));
                    }
                }
            }
            Err(e) => {
                // Desconectou — decrementa antes de qualquer decisão
                connected_count.fetch_sub(1, Ordering::Relaxed);

                let is_stream_error = *has_received_data.lock().unwrap();
                if is_stream_error {
                    let timestamp = now_iso();
                    console.print(&format!(
                        "[dim]{}[/] [dim]↻ Reconectando ao tópico \"{}\"...[/]",
                        timestamp, topic
                    ));
                } else {
                    let timestamp = now_iso();
                    console.print(&format!(
                        "[dim]{}[/] [yellow]⚠[/] Falha ao conectar no tópico \"{}\": {}, reconectando em {}s...",
                        timestamp, topic, e, reconnect_delay
                    ));
                    thread::sleep(Duration::from_secs(reconnect_delay));
                }
            }
        }
    }
}
