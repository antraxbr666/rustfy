mod config;
mod ntfy;

use config::Config;
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use rich_rust::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

static ICON_ENABLED_PNG: &[u8] = include_bytes!("../assets/icon-enabled.png");
static ICON_DISABLED_PNG: &[u8] = include_bytes!("../assets/icon-disabled.png");

fn extract_embedded_assets() -> PathBuf {
    let tmp = PathBuf::from("/tmp").join(format!("rustfy-assets-{}", process::id()));
    fs::create_dir_all(&tmp).expect("Falha ao criar diretório temporário de assets");

    fs::write(tmp.join("icon-enabled.png"), ICON_ENABLED_PNG).expect("Falha ao extrair icon-enabled.png");
    fs::write(tmp.join("icon-disabled.png"), ICON_DISABLED_PNG).expect("Falha ao extrair icon-disabled.png");

    tmp
}

fn main() {
    let console = rich_rust::Console::new();
    console.print(&format!("[bold cyan]RustFy {}[/]", env!("CARGO_PKG_VERSION")));
    console.print("[dim]Iniciando aplicação residente...[/]");

    let (cfg, cfg_path) = Config::load_or_create();

    // Catppuccin Mocha palette
    let mauve = Color::parse("#CBA6F7").unwrap();
    let base = Color::parse("#1E1E2E").unwrap();
    let text = Color::parse("#CDD6F4").unwrap();
    let surface0 = Color::parse("#313244").unwrap();
    let surface2 = Color::parse("#585B70").unwrap();

    let header_style = Style::new().bold().color(base).bgcolor(mauve);
    let cell_style = Style::new().color(text);
    let alt_row_style = Style::new().bgcolor(surface0);

    let mut table = Table::new()
        .border_style(Style::new().color(surface2))
        .header_style(header_style.clone());

    table.add_column(Column::new("Configuração").header_style(header_style.clone()).style(cell_style.clone()));
    table.add_column(Column::new("Valor").header_style(header_style).style(cell_style));

    table.add_row(Row::new(vec![
        Cell::new("Config path"),
        Cell::new(cfg_path.to_string_lossy().to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("base-url"),
        Cell::new(cfg.base_url.clone()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("topics"),
        Cell::new(format!("{:?}", cfg.topics)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("reconnect_delay"),
        Cell::new(format!("{}s", cfg.reconnect_delay)),
    ]));

    let table = table.row_styles(vec![Style::new(), alt_row_style]);

    console.print_renderable(&table);

    gtk::init().expect("Falha ao inicializar GTK");

    let connected_count = Arc::new(AtomicUsize::new(0));

    for topic in &cfg.topics {
        let cc = Arc::clone(&connected_count);
        let base = cfg.base_url.clone();
        let t = topic.clone();
        let delay = cfg.reconnect_delay;
        thread::spawn(move || {
            ntfy::listen_topic(base, t, delay, cc);
        });
    }

    let assets = extract_embedded_assets();
    let assets_str = assets.canonicalize()
        .unwrap_or_else(|_| assets.clone())
        .to_string_lossy()
        .into_owned();

    let mut indicator = AppIndicator::new("rustfy", "icon-disabled");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_icon_theme_path(&assets_str);
    indicator.set_icon_full("icon-disabled", "Rustfy Tray");

    let mut menu = gtk::Menu::new();

    let assets_clone = assets.clone();
    let item_about = gtk::MenuItem::with_label("Sobre");
    item_about.connect_activate(move |_| {
        let about = gtk::AboutDialog::new();
        about.set_program_name("RustFy");
        about.set_version(Some(env!("CARGO_PKG_VERSION")));
        about.set_comments(Some("Notificações em tempo real do ntfy.sh diretamente na área de trabalho.\nEscuta tópicos configurados e exibe notificações nativas via libnotify."));
        about.set_authors(&["antraxbr666@proton.me"]);
        about.set_website(Some("https://github.com/antraxbr666/rustfy"));
        about.set_website_label(Some("GitHub"));
        about.set_license_type(gtk::License::MitX11);
        about.set_position(gtk::WindowPosition::Center);
        about.set_modal(true);

        // Logo com 40% do tamanho original
        if let Ok(pixbuf) = gdk_pixbuf::Pixbuf::from_file(assets_clone.join("icon-enabled.png")) {
            let w = (pixbuf.width() as f64 * 0.4) as i32;
            let h = (pixbuf.height() as f64 * 0.4) as i32;
            if let Some(scaled) = pixbuf.scale_simple(w, h, gdk_pixbuf::InterpType::Bilinear) {
                about.set_logo(Some(&scaled));
            }
        }

        about.connect_response(|dialog, _response| {
            dialog.hide();
        });

        about.show_all();
    });
    menu.append(&item_about);

    let item_quit = gtk::MenuItem::with_label("Sair");
    item_quit.connect_activate(|_| {
        gtk::main_quit();
    });
    menu.append(&item_quit);

    indicator.set_menu(&mut menu);
    menu.show_all();

    let indicator_rc = Rc::new(RefCell::new(indicator));
    let current_icon_name = Rc::new(RefCell::new(String::from("icon-disabled")));

    let indicator_rc_poll = Rc::clone(&indicator_rc);
    let current_icon_name_poll = Rc::clone(&current_icon_name);
    let connected_count_poll = Arc::clone(&connected_count);

    gtk::glib::timeout_add_local(Duration::from_millis(500), move || {
        let count = connected_count_poll.load(Ordering::Relaxed);
        let desired = if count > 0 { "icon-enabled" } else { "icon-disabled" };
        let mut current = current_icon_name_poll.borrow_mut();
        if *current != desired {
            indicator_rc_poll.borrow_mut().set_icon_full(desired, "Rustfy Tray");
            *current = desired.to_string();
        }
        gtk::glib::ControlFlow::Continue
    });

    console.print("[green]✓[/] Indicador configurado. Entrando no loop GTK...\n");

    gtk::main();
}
