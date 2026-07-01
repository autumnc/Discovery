use ratatui::style::{Color, Modifier, Style};

/// Dark theme color palette for hipda-tui
pub struct Theme;

impl Theme {
    pub const BG: Color = Color::Rgb(24, 24, 27);
    pub const SURFACE: Color = Color::Rgb(39, 39, 42);
    pub const BORDER: Color = Color::Rgb(63, 63, 70);
    pub const BORDER_ACTIVE: Color = Color::Rgb(82, 82, 91);

    pub const ACCENT: Color = Color::Rgb(6, 182, 212);       // cyan-500
    pub const ACCENT_DIM: Color = Color::Rgb(14, 116, 144);  // cyan-700
    pub const BLUE: Color = Color::Rgb(59, 130, 246);        // blue-500
    pub const GREEN: Color = Color::Rgb(34, 197, 94);        // green-500
    pub const YELLOW: Color = Color::Rgb(234, 179, 8);       // yellow-500
    pub const RED: Color = Color::Rgb(239, 68, 68);          // red-500
    pub const MAGENTA: Color = Color::Rgb(168, 85, 247);    // purple-500

    pub const TEXT: Color = Color::Rgb(228, 228, 231);       // zinc-200
    pub const TEXT_MUTED: Color = Color::Rgb(161, 161, 170); // zinc-400
    pub const TEXT_DIM: Color = Color::Rgb(113, 113, 122);   // zinc-500
    pub const TEXT_SUBTLE: Color = Color::Rgb(82, 82, 91);   // zinc-600

    // Common styles
    pub fn accent() -> Style { Style::default().fg(Self::ACCENT) }
    pub fn accent_bold() -> Style { Style::default().fg(Self::ACCENT).add_modifier(Modifier::BOLD) }
    pub fn text() -> Style { Style::default().fg(Self::TEXT) }
    pub fn text_dim() -> Style { Style::default().fg(Self::TEXT_DIM) }
    pub fn text_muted() -> Style { Style::default().fg(Self::TEXT_MUTED) }
    pub fn blue() -> Style { Style::default().fg(Self::BLUE) }
    pub fn green() -> Style { Style::default().fg(Self::GREEN) }
    pub fn yellow() -> Style { Style::default().fg(Self::YELLOW) }
    pub fn red() -> Style { Style::default().fg(Self::RED) }
    pub fn magenta() -> Style { Style::default().fg(Self::MAGENTA) }
    pub fn cyan() -> Style { Style::default().fg(Self::ACCENT) }

    pub fn selected() -> Style { Style::default().fg(Self::BG).bg(Self::ACCENT) }
    pub fn selected_dim() -> Style { Style::default().fg(Self::ACCENT).bg(Self::SURFACE) }

    pub fn block() -> Style { Style::default().fg(Self::BORDER) }
    pub fn block_active() -> Style { Style::default().fg(Self::BORDER_ACTIVE) }

    pub fn tab_active() -> Style { Style::default().fg(Self::BG).bg(Self::ACCENT) }
    pub fn tab_inactive() -> Style { Style::default().fg(Self::TEXT_MUTED).bg(Self::SURFACE) }

    pub fn status_normal() -> Style { Style::default().fg(Self::TEXT_DIM) }
    pub fn status_error() -> Style { Style::default().fg(Self::RED) }
}
