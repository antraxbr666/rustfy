use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

fn default_base_url() -> String {
    "https://ntfy.sh".to_string()
}

fn default_reconnect_delay() -> u64 {
    10
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            topics: Vec::new(),
            reconnect_delay: default_reconnect_delay(),
        }
    }
}

impl Config {
    pub fn load_or_create() -> (Self, PathBuf) {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = PathBuf::from(home).join(".config").join("rustfy");
        let config_path = config_dir.join("config.toml");

        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("Aviso: não foi possível criar diretório de configuração ({e}). Usando diretório atual.");
        }

        let config = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match toml::from_str::<Config>(&contents) {
                        Ok(cfg) => cfg,
                        Err(e) => {
                            eprintln!("Aviso: config.toml malformado ou com campos inválidos ({e}). Usando valores padrão.");
                            Config::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Aviso: não foi possível ler config.toml ({e}). Usando valores padrão.");
                    Config::default()
                }
            }
        } else {
            Config::default()
        };

        // Garante que o arquivo no disco tenha todos os campos atualizados
        if let Err(e) = Self::save(&config_path, &config) {
            eprintln!("Erro ao salvar config.toml: {e}");
        }

        (config, config_path)
    }

    fn save(path: &PathBuf, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(config)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}
