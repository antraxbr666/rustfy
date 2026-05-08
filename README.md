<div align="center">

# 🔥 RustFy

> Notificações em tempo real do ntfy.sh diretamente na área de trabalho Linux.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg)
![GTK](https://img.shields.io/badge/GTK-3.24-green.svg)
![Author](https://img.shields.io/badge/author-antraX-purple.svg)

</div>

---

## 📖 Sobre

**RustFy** é um aplicativo residente escrito em Rust que monitora tópicos do [ntfy.sh](https://ntfy.sh) via SSE (Server-Sent Events) e exibe notificações nativas na área de trabalho Linux usando `libnotify`.

Ele roda silenciosamente na bandeja do sistema (system tray), conectando-se automaticamente aos seus tópicos configurados e alertando você sempre que uma nova mensagem chegar — com suporte completo a título, prioridade, tags (emojis) e ícones personalizados.

---

## ✨ Funcionalidades

- 🔔 **Notificações nativas** via D-Bus (`notify-rust`)
- 📡 **Streaming SSE** em tempo real com HTTP/2 (libcurl)
- 🔄 **Auto-reconexão inteligente** com delay configurável
- 🖼️ **Cache de ícones** de URLs com limpeza automática (>2 dias)
- 🎨 **Ícone dinâmico na bandeja**: verde quando conectado, cinza quando desconectado
- ⚙️ **Configuração auto-recuperável** em `~/.config/rustfy/config.toml`
- 📋 **AboutDialog** com informações do projeto, autor e licença MIT
- 🚀 **Binário único** com assets embutidos via `include_bytes!`

---

## 🛠️ Tecnologias

- [Rust](https://www.rust-lang.org/) — linguagem principal (edition 2024)
- [GTK3](https://gtk.org/) — toolkit gráfico para o tray e dialog
- [libappindicator](https://github.com/AyatanaIndicators/libappindicator) — ícone na bandeja do sistema
- [libcurl](https://curl.se/libcurl/) — streaming HTTP/2 SSE
- [notify-rust](https://github.com/hoodie/notify-rust) — notificações nativas via D-Bus
- [rich_rust](https://crates.io/crates/rich_rust) — output colorido no terminal (Catppuccin Mocha)

---

## 📦 Pré-requisitos

- **Rust 1.85+** (necessário para `edition = "2024"`)
- **GTK3 / AppIndicator** (bibliotecas do sistema)
- **X11 ou Wayland** (ambiente gráfico)

### Instalação das libs no sistema

**Debian/Ubuntu:**
```bash
sudo apt install libgtk-3-dev libappindicator3-dev
```

**Fedora:**
```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel
```

---

## 🚀 Instalação

```bash
# 1. Clone o repositório
git clone https://github.com/antraxbr666/rustfy.git
cd rustfy

# 2. Compile em release
cargo build --release

# 3. Execute
./target/release/rustfy
```

---

## ⚙️ Configuração

Na primeira execução, o RustFy cria automaticamente o arquivo:

```
~/.config/rustfy/config.toml
```

Exemplo de configuração:

```toml
base_url = "https://ntfy.sh"
topics = ["alertas", "servidor"]
reconnect_delay = 10
```

| Campo | Descrição |
|-------|-----------|
| `base_url` | URL do servidor ntfy |
| `topics` | Lista de tópicos para escutar |
| `reconnect_delay` | Delay (segundos) entre reconexões |

O arquivo é **auto-recuperável**: campos ausentes são preenchidos com padrões e o arquivo é reescrito a cada inicialização.

---

## 📚 Uso

Após iniciar, o RustFy aparece na bandeja do sistema com o ícone do projeto.

### Ícone do tray
- 🟢 **Verde** — conectado ao servidor ntfy
- ⚪ **Cinza** — desconectado ou tentando reconectar

### Menu do tray (clique direito)
- **Sobre** — abre o diálogo com informações do projeto
- **Sair** — encerra o aplicativo

### Enviar uma notificação de teste

```bash
curl -H "X-Title: Teste" \
     -H "X-Priority: 5" \
     -H "X-Icon: https://ntfy.sh/static/img/ntfy.png" \
     -d "Mensagem de teste do RustFy!" \
     https://ntfy.sh/alertas
```

---

## 🤝 Contribuindo

Contribuições são bem-vindas! Abra uma issue ou envie um PR.

1. Fork o projeto
2. Crie sua branch: `git checkout -b feature/minha-feature`
3. Commit suas mudanças: `git commit -m 'feat: adiciona minha feature'`
4. Push: `git push origin feature/minha-feature`
5. Abra um Pull Request

---

## ☠ Autor

**antraX**
- 📧 Email: [antraxbr666@proton.me](mailto:antraxbr666@proton.me)
- 🐙 GitHub: [@antraxbr666](https://github.com/antraxbr666)

---

## 📄 Licença

```
MIT License

Copyright (c) 2026 antraX

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
