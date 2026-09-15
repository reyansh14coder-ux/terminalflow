#![allow(dead_code)]

use ratatui::style::{Color, Style, Modifier};

pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub text: Color,
    pub text_dim: Color,
    pub background: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Magenta,
            accent: Color::Yellow,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            text_dim: Color::Gray,
            background: Color::Black,
        }
    }

    pub fn dark() -> Self {
        Self::default()
    }

    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Magenta,
            accent: Color::DarkGray,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::Black,
            text_dim: Color::Gray,
            background: Color::White,
        }
    }

    pub fn cyberpunk() -> Self {
        Self {
            primary: Color::Magenta,
            secondary: Color::Cyan,
            accent: Color::Yellow,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            text_dim: Color::Gray,
            background: Color::Black,
        }
    }

    pub fn ocean() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Yellow,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            text_dim: Color::Gray,
            background: Color::Black,
        }
    }

    pub fn forest() -> Self {
        Self {
            primary: Color::Green,
            secondary: Color::Cyan,
            accent: Color::Yellow,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            text_dim: Color::Gray,
            background: Color::Black,
        }
    }

    pub fn style_primary(&self) -> Style {
        Style::default().fg(self.primary)
    }

    pub fn style_secondary(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn style_accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn style_success(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn style_warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn style_error(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn style_text(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn style_dim(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    pub fn style_bold(&self) -> Style {
        Style::default().fg(self.text).add_modifier(Modifier::BOLD)
    }
}
