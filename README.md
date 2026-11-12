# Vsrmi

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)

> **Vsrmi** — VS Code benzeri akıllı tamamlama ve otomatik parantez kapatma ile donatılmış, terminal içinde çalışan modal kod editörü.  
> [Termi] projesinden çatallanmıştır. Rust, [ratatui] ve [crossterm] ile inşa edilmiştir.

---

## ✨ Özellikler

| Özellik | Açıklama |
|---|---|
| 🔤 **Modal Düzenleme** | Normal / Insert / Visual / Command modları, vi-tarzı hareketler |
| 🔍 **Arama** | Artımlı, literal veya regex, akıllı büyük/küçük harf |
| 🎨 **Sözdizimi Vurgulama** | Rust, C, C++, Zig, Python, Markdown |
| 💡 **Akıllı Tamamlama** | Ghost-text; keyword, type, constant ve yerel kelime tahmini |
| 🔒 **Otomatik Çift Kapatma** | `()` `[]` `{}` `""` `''` otomatik kapanır, Backspace ikisini siler |
| 📁 **Çoklu Buffer** | Sekme şeridi, dosya ağacı (`Ctrl+B`), atomik kayıt |
| 🎭 **Temalar** | Yerleşik koyu/açık + TOML ile özelleştirme |

---

## 📦 Kurulum

### Gereksinimler

- **Rust ≥ 1.88** — [rustup.rs](https://rustup.rs) üzerinden kurabilirsiniz:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version   # rustc 1.88.x veya üzeri olmalı
```

> **Not:** Özel bir font gerekmez. Vsrmi yalnızca standart Unicode blok karakterleri kullanır — her modern terminal emülatöründe çalışır.

---

### Yöntem 1 — Git'ten Doğrudan Kur

```sh
cargo install --git https://github.com/Lefkee/Vsrmi vsrmi
```

Derleme tamamlandığında `vsrmi` komutu `~/.cargo/bin/` altına kurulur ve hemen kullanılabilir hale gelir.

---

### Yöntem 2 — Kaynak Koddan Derle

```sh
# 1. Depoyu klonla
git clone https://github.com/Lefkee/Vsrmi
cd Vsrmi

# 2. Optimizasyonlu derle
cargo build --release

# 3. Çalıştır
./target/release/vsrmi dosyaadi.py
```

**Sisteme kalıcı olarak eklemek için:**

```sh
# Kopyalama yöntemi:
sudo cp target/release/vsrmi /usr/local/bin/vsrmi

# veya Cargo ile:
cargo install --path .
```

---

### Dil Desteği

Vsrmi, dosya uzantısından dili otomatik algılar:

| Uzantı | Dil |
|---|---|
| `.rs` | Rust |
| `.c` `.h` | C |
| `.cpp` `.hpp` `.cc` | C++ |
| `.zig` | Zig |
| `.py` | Python |
| `.md` | Markdown |

```sh
vsrmi main.py        # Python modu ile açar
vsrmi src/main.rs    # Rust modu ile açar
```

---

## 🖥️ Arayüz

### Status Bar

Ekranın altındaki durum çubuğu soldan sağa şu bilgileri gösterir:

```
◆ NORMAL ▌ dosyaadi.py ●        python  LF  Ln 12 Col 8  ▅42%
```

| Gösterge | Açıklama |
|---|---|
| `◆ NORMAL` | Aktif mod — rengi moda göre değişir |
| `▌` | Mod/dosya adı ayracı |
| `●` | Kaydedilmemiş değişiklik |
| `▁▃▅▆▇█` | Dosyadaki kaydırma konumu (%) |

**Mod renkleri:**

| Mod | Gösterge | Renk |
|---|---|---|
| Normal | `◆ NORMAL` | Mavi |
| Insert | `▶ INSERT` | Yeşil |
| Visual | `▪ VISUAL` | Mor |
| Command | `: COMMAND` | Şeftali |
| Search | `/ SEARCH` | Sarı |
| Tree | `⊞ TREE` | Gök mavisi |

### Dosya Ağacı (`Ctrl+B`)

Dosya ağacında uzantı etiketleri gösterilir:

| Etiket | Uzantı |
|---|---|
| `[rs]` | Rust |
| `[py]` | Python |
| `[md]` | Markdown |
| `[c+]` | C++ |
| `[cf]` | TOML/YAML |
| `[sh]` | Shell |

---

## ⌨️ Tuş Referansı

> Editörün içinde `:help` yazarak tam listeye ulaşabilirsiniz.

### Modlar

| Tuş | İşlev |
|---|---|
| `i` `a` `I` `A` `o` `O` | Insert moduna gir |
| `v` `V` | Karakter / satır Visual modu |
| `Esc` | Normal moda dön, çoklu imleci kapat |

### Hareketler

| Tuş | İşlev |
|---|---|
| `h j k l` | Sol / Aşağı / Yukarı / Sağ |
| `w b e` | Kelime ileri / geri / sonu |
| `0` `^` `$` | Satır başı / ilk karakter / sonu |
| `gg` `G` | Dosya başı / sonu |

### Düzenleme

| Tuş | İşlev |
|---|---|
| `x` | Karakter sil |
| `dd` `yy` `p` | Satır sil / kopyala / yapıştır |
| `u` `Ctrl+R` | Geri al / yeniden yap |
| `Tab` *(Insert modda)* | Otomatik tamamlamayı kabul et |
| `Alt+↑` `Alt+↓` | Çoklu imleç ekle |

### Dosya & Navigasyon

| Tuş | İşlev |
|---|---|
| `Ctrl+S` | Kaydet |
| `Ctrl+Q` | Çık |
| `Ctrl+B` | Dosya ağacını aç/kapat |
| `Ctrl+N` `Ctrl+P` | Sonraki / önceki buffer |
| `/` `?` `n` `N` | İleri / geri ara, tekrarla |

### Komut Satırı

```
:w [yol]           kaydet
:q[!]              çık (! ile kaydetmeden)
:wq                kaydet ve çık
:e[!] yol          dosya aç (! ile kaydetmeden)
:bn / :bp          sonraki / önceki buffer
:<satır>           satıra atla
:set <ayar>        ayar değiştir
:theme <isim>      tema değiştir
:%s/desen/yeni/g   toplu değiştir
```

---

## ⚙️ Yapılandırma

Yapılandırma dosyasını ilgili konuma kopyalayın:

| Platform | Konum |
|---|---|
| Linux | `~/.config/vsrmi/config.toml` |
| macOS | `~/Library/Application Support/vsrmi/config.toml` |
| Windows | `%APPDATA%\vsrmi\config.toml` |

`VSRMI_CONFIG_DIR` ortam değişkeni bu konumu tamamen geçersiz kılar.

Başlangıç noktası olarak [`config.example.toml`](config.example.toml) dosyasını kullanın. Temalar `<config-dir>/vsrmi/themes/<isim>.toml` konumuna yerleştirilir:

```toml
name = "midnight"
base = "dark"

[comment]
fg = "#4a5058"
italic = true

[selection]
bg = "bright-blue"
```

---

## 🏗️ Mimari

Katmanlar yalnızca aşağıya bağımlıdır:

```
app/           olay döngüsü, durum, aksiyon dispatch, ex komutları
├── ui/        düzen ve widget'lar; renderer/ terminale sahip
├── input/     tuşlar → aksiyonlar
└── editor/    metin; terminal bilgisi yok
    ├── document/  rope, dosya, kirli durum, girinti
    ├── cursor/    pozisyonlar, hareketler, kelime sınırları
    ├── selection/ karakter aralıkları
    ├── buffer/    belge + imleçler + görüntü + geçmiş
    └── command/   ex-komut ayrıştırma

config/  theme/  syntax/  search/  undo/  clipboard/  filesystem/
```

`unsafe_code` tüm crate genelinde yasaktır.

---

## 🔧 Geliştirme

```sh
cargo test                                  # birim testleri
cargo clippy --all-targets -- -D warnings  # lint
cargo fmt --all                             # formatlama
```

---

## 📄 Lisans

MIT — Ayrıntılar için [LICENSE](LICENSE) dosyasına bakın.

[ratatui]: https://ratatui.rs
[crossterm]: https://github.com/crossterm-rs/crossterm
[Termi]: https://github.com/tuna4ll/termi
[releases]: https://github.com/Lefkee/Vsrmi/releases
[CONTRIBUTING.md]: CONTRIBUTING.md

