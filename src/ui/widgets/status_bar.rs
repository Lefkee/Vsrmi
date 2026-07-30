//! # Status Bar (Durum Çubuğu)
//!
//! **Amacımız:** Kullanıcıya o an hangi dosyada olduğunu ve dosyanın ne durumda olduğunu şipşak göstermek.
//!
//! **Ne İş Yapar?:** Ekranın en altındaki o tek satırlık çubuktur. Sol tarafta (aktif mod, dosya adı, 
//! kaydedilmemiş değişiklik uyarısı) ve sağ tarafta (dil, imleç konumu, satır sonu tipi,
//! sayfayı ne kadar kaydırdığımızın yüzdesi) gibi bilgileri barındırır. Her modun kendine has
//! tatlı bir vurgu rengi vardır, böylece editörün hangi modda olduğunu ta uzaklardan bile bir bakışta anlayabilirsiniz.
//!
//! **Dışarıya Açık Yapılar:** [`StatusBar`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::app::mode::Mode;
use crate::editor::cursor::Position;
use crate::editor::document::LineEnding;
use crate::theme::Theme;

/// Metin alanının en altındaki bilgi çubuğumuzu temsil eden yapı.
pub struct StatusBar<'a> {
    /// O an bulunduğumuz mod (renkli ve dikkat çekici bir etiket olarak gösterilir).
    pub mode: Mode,
    /// Üzerinde çalıştığımız dosyanın adı. Eğer henüz kaydedilmemiş bir dosyadaysak `[No Name]` yazar.
    pub name: &'a str,
    /// Dosyada henüz kaydetmediğimiz taze değişiklikler var mı?
    pub dirty: bool,
    /// Dosyanın otomatik algılanan dili (bulunamazsa `plain` olur).
    pub language: &'a str,
    /// Ana imlecimizin anlık konumu.
    pub position: Position,
    /// Scroll (kaydırma) yüzdesini hesaplayabilmek için dosyadaki toplam satır sayısı.
    pub line_count: usize,
    /// Dosyayı kaydederken kullanılacak satır sonu karakteri (LF veya CRLF).
    pub line_ending: LineEnding,
    /// Ekrandaki aktif imleç sayısı. (Eğer birden fazla imleç varsa gösterilir, tekse gizlenir).
    pub cursor_count: usize,
    /// Arayüzümüzün renk ve tema ayarları.
    pub theme: &'a Theme,
}

impl StatusBar<'_> {
    /// Her moda özel vurgu rengini belirliyoruz ki, mod etiketimiz o anki duruma göre
    /// farklı ve tatlı renklerle parlasın.
    fn mode_style(&self) -> Style {
        let bg = match self.mode {
            Mode::Normal     => Color::Rgb(0x89, 0xb4, 0xfa), // blue
            Mode::Insert     => Color::Rgb(0xa6, 0xe3, 0xa1), // green
            Mode::Visual
            | Mode::VisualLine => Color::Rgb(0xcb, 0xa6, 0xf7), // mauve
            Mode::Command    => Color::Rgb(0xfa, 0xb3, 0x87), // peach
            Mode::Search     => Color::Rgb(0xf9, 0xe2, 0xaf), // yellow
            Mode::Tree       => Color::Rgb(0x89, 0xdc, 0xeb), // sky
        };
        Style::new()
            .fg(Color::Rgb(0x1e, 0x1e, 0x2e))
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    }

    /// O anki modu temsil eden kısa ve öz etiket metni.
    const fn mode_label(mode: Mode) -> &'static str {
        match mode {
            Mode::Normal     => " ◆ NORMAL ",
            Mode::Insert     => " ▶ INSERT ",
            Mode::Visual     => " ▪ VISUAL ",
            Mode::VisualLine => " ▪ V-LINE ",
            Mode::Command    => " : COMMAND",
            Mode::Search     => " / SEARCH ",
            Mode::Tree       => " ⊞ TREE   ",
        }
    }

    /// İmlecimizin dosyanın yüzde kaçlık bir kısmına denk geldiğini hesaplıyoruz.
    /// (Matematiksel olarak ufak bir bölme işlemi yapıp yüzdelik değere çeviriyoruz.)
    fn scroll_percentage(&self) -> usize {
        let last = self.line_count.saturating_sub(1);
        (self.position.line * 100).checked_div(last).unwrap_or(100)
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() {
            return;
        }

        let base = self.theme.status;
        surface.set_style(area, base);

        // ── Sol Kısım ─────────────────────────────────────────────────────────
        // Buraya sırasıyla şunları diziyoruz: [MOD ETİKETİ] [Ayraç] [Dosya Adı] [Kaydedilmemişlik Noktası] [Çoklu İmleç Bilgisi]
        let mode_style = self.mode_style();
        let mut left = vec![
            Span::styled(Self::mode_label(self.mode), mode_style),
            // Mod etiketi ile dosya adı arasındaki blok tarzı şık ayracımız.
            Span::styled("▌", mode_style.bg(base.bg.unwrap_or(Color::Reset)).fg(mode_style.bg.unwrap_or(Color::Reset))),
            Span::styled(format!("  {} ", self.name), base),
        ];

        if self.dirty {
            left.push(Span::styled("● ", self.theme.status_dirty));
        }
        if self.cursor_count > 1 {
            left.push(Span::styled(
                format!(" ×{}", self.cursor_count),
                base.add_modifier(Modifier::ITALIC),
            ));
        }

        // ── Sağ Kısım ─────────────────────────────────────────────────────────
        // Ve sağ tarafa da şunları hizalıyoruz: [Dil] [Satır Sonu] [Satır:Sütun] [%]
        let pct = self.scroll_percentage();
        let scroll_icon = match pct {
            0          => "▁",
            1..=24     => "▃",
            25..=49    => "▅",
            50..=74    => "▆",
            75..=99    => "▇",
            _          => "█",
        };
        let right = Line::from(Span::styled(
            format!(
                " {}  {}  {}  Ln {} Col {}  {}{}% ",
                self.language,
                self.line_ending.label(),
                if self.cursor_count > 1 { format!("{} cursors", self.cursor_count) } else { String::new() },
                self.position.line + 1,
                self.position.col + 1,
                scroll_icon,
                pct,
            ),
            base,
        ))
        .right_aligned();

        Line::from(left).render(area, surface);
        right.render(area, surface);
    }
}
