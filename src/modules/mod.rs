use colorful::{Color, Colorful, Style};
use std::{
    borrow::Cow,
    fmt::{self, Display},
};

mod ps1;
pub(crate) use ps1::Ps1;

mod ps2;
pub(crate) use ps2::Ps2;

type ModuleOut = Option<ModuleInfo>;

#[derive(Debug, Default)]
struct ModuleInfo {
    icon: Option<&'static str>,
    info: Option<Cow<'static, str>>,
    color: Option<Color>,
    style: Option<Style>,
}

impl ModuleInfo {
    fn of() -> Self {
        Self::default()
    }

    fn icon(self, icon: &'static str) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }

    fn info(self, info: impl Into<Cow<'static, str>>) -> Self {
        Self {
            info: Some(info.into()),
            ..self
        }
    }

    fn text(self, text: impl ToString) -> Self {
        self.info(text.to_string())
    }

    fn color(self, color: Color) -> Self {
        Self {
            color: Some(color),
            ..self
        }
    }

    fn style(self, style: Style) -> Self {
        Self {
            style: Some(style),
            ..self
        }
    }

    fn some(self) -> ModuleOut {
        Some(self)
    }
}

impl Display for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            todo!("explain")
        } else {
            if let Some(icon) = self.icon {
                if self.info.is_some() {
                    write!(f, "{} ", icon)?
                } else {
                    write!(f, "{}", icon)?
                }
            }
            if let Some(info) = &self.info {
                match (self.color, self.style) {
                    (None, None) => write!(f, "{}", info),
                    (Some(c), None) => write!(f, "{}", info.color(c)),
                    (None, Some(s)) => write!(f, "{}", info.style(s)),
                    (Some(c), Some(s)) => write!(f, "{}", info.color(c).style(s)),
                }?
            }
        }
        Ok(())
    }
}
