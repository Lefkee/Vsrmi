# Vsrmi 0.1.0 — Kapsamlı Yenileme Tasarımı

**Tarih:** 2026-11-12
**Durum:** Onay bekliyor
**Seçilen yaklaşım:** A — Görünür yenileme

## Amaç

Vsrmi'yi ilk kez gören birinin "olgun, bakımlı ve günlük kullanılabilir bir terminal
editörü" izlenimi almasını sağlamak. Bunu yaparken mevcut 118 testlik davranış
sözleşmesini bozmamak ve riskli yeni durum yönetimi (oturum kalıcılığı, yeni mod)
eklememek.

## Neden şimdi

Proje çalışıyor ama vitrin tarafı eksik: boş açılışta ekran tamamen boş, README
sürüm ve CI durumunu göstermiyor, changelog hâlâ çatallandığı depoyu işaret ediyor,
sürüm numarası `0.0.1` ile changelog'daki `0.1.0` çelişiyor. Bağımlılıklarda
`dirs` 6 ve `arboard` 3.6 geride kalmış; `cargo update` ayrıca yanked bir
`clipboard-win` sürümünü düzeltiyor.

## Hedefler

- Boş açılışta yönlendirici bir welcome ekranı.
- TOML, JSON, YAML ve Shell desteği; dosya ağacındaki mevcut `[cf]`/`[sh]`
  etiketleriyle tutarlılık.
- Durum çubuğunda dar terminal ve tek imleç durumlarının düzeltilmesi; modül
  dokümantasyonunun İngilizceye çevrilmesi.
- Yardım popup'ının gerçek kısayol setini kapsaması ve sürüm göstermesi.
- `dirs` 7, `arboard` 3.7 ve MSRV 1.88 ile uyumlu `cargo update`.
- Sürüm 0.1.0; changelog'un tek ilk sürüm altında birleştirilmesi ve bağlantıların
  `Lefkee/Vsrmi` olması.
- README'de CI/lisans/Rust badge'leri, dürüst metin önizleme, güncel dil tablosu.
- Dependabot ve CI'da `cargo package` kontrolü.

## Hedef olmayanlar

- Komut paleti (`Ctrl+Shift+P`), son açılan dosyalar, oturum geri yükleme.
- Yeni modal durum veya yeni kalıcı dosya formatı.
- Tree-sitter'a geçiş; regex motoru korunur.
- Etiket (`v0.1.0`) veya crates.io yayını tetiklemek. Release workflow yalnızca
  etiket push edilirse çalışır; bu yenilemede etiket push edilmez.

## Anti-hedefler

- Welcome ekranının tuş yemesini engellemek: hiçbir tuş yutulmaz. Oturum Normal
  modda başlar; `i` ile Insert'e geçilir ve ilk metin girildiğinde welcome
  kendiliğinden kaybolur. Ekranı gizlemek için ayrı bir "herhangi bir tuş" durumu
  eklenmez.
- Sahte ekran görüntüsü koymak: kayıt aracı (vhs/asciinema/tmux) yok; README'de
  gerçek çıktıya dayanan metin önizleme kullanılacak.
- Kararsız bağımlılıklara geçmek: `notify` 9 RC ve `ropey` 2 beta; kararlı
  sürümlerde kalınır.
- Davranış değiştiren sessiz refactor: yeni özellikler testle, mevcut davranış
  regresyon testleriyle korunur.

## Kısıtlar

- Rust 2024, `rust-version = "1.88"`, `unsafe_code = "forbid"`.
- UI katmanı yalnızca çizer; mutasyon yalnızca viewport kaydırma ve sözdizimi
  önbelleği genişletmedir. Welcome bu kurala uyar.
- Tema renkleri yalnızca `theme` modülünde tanımlanır. Welcome yeni tema alanı
  istemez; mevcut `text`, `gutter`, `syntax.keyword`, `syntax.function`,
  `popup_border` slotlarını kullanır.
- Her yeni davranış TDD ile: önce kırmızı test, sonra üretim kodu.
- Doğrulama: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo build --release --locked`, `cargo package --locked`.

## Yaklaşımlar

### A — Görünür yenileme (seçildi)

Welcome ekranı + 4 dil + durum/yardım cilası + bağımlılık/sürüm/sunum/CI.
Görünür etki yüksek, yeni kalıcı durum yok, regresyon yüzeyi sınırlı.

### B — Günlük kullanım paketi

A + son dosyalar, oturum geri yükleme, komut paleti. Daha fazla özellik; ancak
kalıcı durum, yeni tuş yolları ve hata senaryoları nedeniyle kapsam yaklaşık iki
katına çıkar.

### C — Sadece bakım

Bağımlılık, changelog, README ve CI. En güvenlisi; fakat projeyi "yeni" göstermez.

**Neden A:** Kullanıcının istediği tüm başlıkları kapsar, B'nin kalıcılık riskini
taşımaz. B ayrı bir sonraki sürümün işi olarak bırakılır.

## Tasarım

### 1. Welcome ekranı

**Koşul.** Yeni ve saf bir metot:

```rust
impl Document {
    /// No path, no text, no unsaved changes: the state shown on a bare start.
    pub fn is_blank_scratch(&self) -> bool;
}
```

`path.is_none() && !self.dirty && self.text.len_chars() == 0`.

- `vsrmi` (dosyasız) → Normal modda welcome görünür.
- `vsrmi yeni.rs` (henüz olmayan dosya) → `path` dolu olduğu için welcome yok.
- `i` Insert'e geçirir; kullanıcı ilk karakteri yazınca metin boş olmaktan çıkar →
  welcome kaybolur. Normal moddaki `j`/`k` gibi hareketler boş belgede welcome'i
  gizlemez, çünkü hiçbir metin üretmezler.
- `:[line]`, `:e`, dosya ağacından açma welcome'i kendiliğinden kapatır.

**Bileşen.** `src/ui/widgets/welcome.rs`, `pub struct Welcome<'a>`:

```text
┌──────────────────────────────────────────────────┐
│                                                  │
│   vsrmi                                v0.1.0    │
│   a modal terminal code editor                   │
│                                                  │
│   i        insert mode                           │
│   :help    every key                             │
│   Ctrl+B   file tree                             │
│   Ctrl+S   save                                  │
│                                                  │
│   Press i to start · :e <file> to open           │
└──────────────────────────────────────────────────┘
```

- `ratatui::widgets::Widget` ile çizilir; `Clear` kullanmaz, editör alanını
  `theme.text` ile doldurur.
- Tam panel 13 satır ve yaklaşık 56 kolondur. Tam düzen yalnızca
  `area.width >= 60 && area.height >= 15` iken çizilir; panel genişliği
  `min(area.width - 4, 56)`, yüksekliği 13'tür.
- `60×15`in altında, `area.width >= 20` ise iki satırlık kompakt ipucu çizilir:
  `vsrmi 0.1.0` ve `i to start · :help · Ctrl+B`.
- `area.width < 20` veya `area.height < 2` ise hiçbir şey çizilmez; editör boş
  görünür. Eşikler ölçülen içerikten türetilir, sabit "küçük ekran" tahmininden
  değil.
- Renkler: başlık `syntax.keyword`, sürüm/ipucu `gutter`, çerçeve `popup_border`,
  gövde `text`. Tema dosyası değişmez.
- `ui::draw` içinde editör çizildikten sonra, tab/status çizilmeden önce
  `regions.editor` alanına çizilir; welcome görünürken imleç gizlenir.
- Sürüm `env!("CARGO_PKG_VERSION")` ile gelir.

**Testler.**
- `Document::is_blank_scratch` doğru/yanlış durumları.
- `Welcome` render testi: `TestBackend` üzerinde "vsrmi", sürüm, `Ctrl+B` ve
  `:e` metinleri görünür.
- Sınır testleri: `60×15` tam panel, `59×15` ve `60×14` kompakt, `20×2` kompakt,
  `19×2` boş; hiçbirinde panik yok.

### 2. Yeni diller

`src/syntax/languages/` altına dört modül ve `all()` sırasına ekleme:

| Dil | Uzantı / dosya | Özel kurallar |
|---|---|---|
| `toml` | `toml`, `Cargo.lock` | `[table]` başlığı, anahtar, tek tırnaklı string, `#` yorum |
| `json` | `json` | anahtar `"..." :`, `true/false/null`, string/number |
| `yaml` | `yaml`, `yml` | anahtar, `---`/`...`, `#` yorum, `&anchor`/`*alias`, `!!tag` |
| `shell` | `sh`, `bash`, `zsh`, `fish`, `ksh`; `.bashrc`, `.zshrc`, `.profile` | `#` yorum, `$VAR`/`${VAR}`/`$?`, atama, fonksiyon tanımı |

- Mevcut `Language` alanları dışında motor değişikliği yok.
- `macro_suffix: false`, `capitalised_types: false`; TOML/YAML/JSON `prose: false`.
- Snippet eklenmez; tamamlama mevcut keyword/type/constant listelerini kullanır.
- Çift tırnak içindeki `"$HOME"` motor tarafından tek bir `String` olarak
  renklendirilir; `$HOME` yalnızca tırnaksız kullanıldığında `Constant` olur.
  Bunun için motor değiştirilmez.
- `README` dil tablosu ve `:help`/status bar otomatik olarak yeni adları gösterir.

**Testler.** `syntax::engine` içindeki `sketch` yardımcısıyla her dil için en az
bir anahtar + string + yorum/yapı testi. Örnek:
- `key = "value" # note` → anahtar `Attribute`, string `String`, yorum `Comment`.
- `{"ok": true}` → anahtar `Attribute`, `true` `Constant`.
- `name: value # note` → anahtar `Attribute`, yorum `Comment`.
- `if [ -n $HOME ]; then` → `if/then` keyword, `$HOME` constant.
- `echo "$HOME"` → tüm çift tırnaklı bölüm string; ayrı constant beklenmez.

### 3. Durum çubuğu

- Modül dokümantasyonu ve alan yorumları İngilizceye çevrilir (diğer modüllerle
  tutarlılık).
- Sağ taraf segment listesi olarak kurulur: dil, satır sonu, varsa çoklu imleç,
  `Ln/Col`, kaydırma yüzdesi. Tek imleçte çift boşluk üreten boş segment kaldırılır.
- Genişlik hesabı: sol ve sağ satır genişlikleri toplamı alanı aşıyorsa sağ taraf
  hiç çizilmez; böylece dar terminalde sağ metin dosya adının üzerine yazmaz.
- Renkler ve mod etiketleri değişmez.

**Testler.** `TestBackend` ile:
- Tek imleçte çıktıda çift boşluk yok.
- 24 kolonluk alanda sağ taraf çizilmez, dosya adı bozulmaz.
- Geniş alanda dil, `LF`, `Ln`, `Col` ve `%` görünür.

### 4. Yardım popup'ı

- Başlık `Vsrmi <sürüm>`.
- İçerik, tüm keymap'in dökümü değil, hatırlanmaya değer seçili kümedir:
  modlar, `h/j/k/l`, `w/b/e`, `0/^/$`, `gg/G`, `x`, `dd/yy/p`, `u`, `Ctrl+R`,
  `Alt+↑/↓`, `Tab` tamamlama, arama, `Ctrl+B`, `Ctrl+N/P`, `Ctrl+S`, `Ctrl+Q`,
  `:w/:q/:wq/:e/:bn/:bp/:set/:theme/:help`.
- Son satıra desteklenen diller eklenir:
  `LANGUAGES rust c cpp zig python markdown toml json yaml shell`.
- Metin 80 kolona sığacak şekilde hizalanır; popup yüksekliği içerikle büyür.
- `HELP` sabiti için birim testi: "LANGUAGES", "toml", "Ctrl+B", "Alt+" içerir.

### 5. Sürüm, changelog ve bağımlılıklar

- `Cargo.toml`: `version = "0.1.0"`.
- `CHANGELOG.md`: `[Unreleased]` boş bırakılır; mevcut `Unreleased` maddeleri ve
  eski `0.1.0` içeriği tek `## [0.1.0] - 2026-11-12` altında birleştirilir. Yeni
  welcome, diller, durum çubuğu ve bağımlılık notları eklenir. Bağlantılar:
  `https://github.com/Lefkee/Vsrmi/compare/v0.1.0...HEAD` ve
  `.../releases/tag/v0.1.0`.
- `Cargo.toml` yorumu: "Edition 2026" → "Edition 2024".
- `.gitattributes`: `Cargo.lock` satırından `-diff` kaldırılır; `linguist-generated`
  kalır. Lock değişiklikleri incelemede görünür olur.
- Bağımlılıklar: `dirs = "7"`, `arboard = "3.7"`, ardından `cargo update`.
  `notify` 8 ve `ropey` 1.6 kararlı sürümlerinde kalır.
- `dirs` 7 MSRV'sini yayınlamıyor. `cargo check` Rust 1.88 ile geçmezse `dirs = "6"`
  korunur ve bu sapma changelog'da "bilinen" olarak yazılır. `arboard` 3.7 MSRV
  1.71 olduğu için güvenli.
- `cargo update`, yanked `clipboard-win 5.4.1` sorununu `5.4.3`e taşır.

### 6. README ve CI

**README (Türkçe kalır).**
- Badge'ler: CI durumu, MIT lisansı, Rust 1.88+, sürüm 0.1.0.
- Kısa tanıtım + welcome ekranının gerçek çıktısına dayanan `text` önizleme.
- Kurulum: git'ten `cargo install --git ... vsrmi`, kaynaktan derleme, ikili yol.
- Dil tablosu güncellenir; `[cf]`/`[sh]` etiketleri artık gerçekten desteklenir.
- Tuş referansı `:help` ile uyumlu hale getirilir.
- Mimari ve geliştirme bölümleri korunur; `Termi` atfı kalır, yalnızca changelog
  bağlantıları düzeltilir.
- README içindeki `[releases]`, `[CONTRIBUTING.md]` bağlantıları kullanılmayan
  tanımlar ise temizlenir veya metne bağlanır.

**CI.**
- `.github/dependabot.yml`: `cargo` ve `github-actions` için haftalık güncelleme.
- `ci.yml`'e `package` işi: `cargo package --locked` (temiz checkout üzerinde).
  Böylece `exclude`, README/lisans varlığı ve paket içeriği her PR'da doğrulanır.

**Release.** Workflow değişmez; etiket push edilmediği için bu yenileme yayın
tetiklemez. Workflow'un `vsrmi` artifact adları zaten düzeltilmiş durumda.

### 7. Git akışı

- Tasarım dokusu `docs/superpowers/specs/2026-11-12-vsrmi-renewal-design.md`
  olarak eklenir.
- Uygulama küçük ve anlamlı commit'lerle `main` üzerinde ilerler; sonunda
  `origin/main`e normal push yapılır. Force-push gerekmez.
- `backup/pre-squash` dalına dokunulmaz.

## Doğrulama

Her aşamadan sonra ve finalde:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release --locked
cargo package --locked --allow-dirty
```

Ek CLI duman testi:

```sh
target/release/vsrmi --version   # Vsrmi 0.1.0
target/release/vsrmi --help      # kullanım metni
```

TUI etkileşimi otomatik test edilemez; welcome/status/widget davranışı
`TestBackend` testleriyle, geri kalanı mevcut regresyon testleriyle doğrulanır.

## Yapıcıya bırakılanlar

- Welcome panelinin tam iç boşlukları ve hizalama sabitleri.
- Yeni dil tanımlarındaki keyword/type/constant listelerinin kapsamı; tabloda
  belirtilen özel kurallar bağlayıcıdır.
- Yardım popup'ının satır sonları ve hizalaması.
- Commit mesajlarının bölünmesi; kullanıcı geçmişinin korunması dışında kural yok.

## Riskler ve geri dönüş

- `cargo update` geniş çaplıdır: testler ve clippy kapıdır; kırılırsa ilgili paket
  tek tek sabitlenir.
- `dirs` 7 MSRV sorunu: derlenmezse 6'ya dönülür.
- Durum çubuğu düzenlemesi görseldir: `TestBackend` testleri ve geniş/dar iki
  senaryo ile korunur.
- Welcome yanlışlıkla gerçek dosyalarda görünürse: koşul yalnızca
  `is_blank_scratch()`; dosya yolu olan her buffer welcome'i kapatır.
