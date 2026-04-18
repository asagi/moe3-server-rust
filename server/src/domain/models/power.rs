// external crates
use strum::Display;
use strum::EnumIter;
use strum::EnumProperty;
use strum::EnumString;
#[cfg(test)]
use strum::IntoEnumIterator;

/// 国の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, EnumString, EnumProperty)]
pub enum Power {
    #[strum(serialize = "Austria", props(Symbol = "a", Adj = "Austrian"))]
    Austria = 1,
    #[strum(serialize = "England", props(Symbol = "e", Adj = "English"))]
    England = 2,
    #[strum(serialize = "France", props(Symbol = "f", Adj = "French"))]
    France = 3,
    #[strum(serialize = "Germany", props(Symbol = "g", Adj = "German"))]
    Germany = 4,
    #[strum(serialize = "Italy", props(Symbol = "i", Adj = "Italian"))]
    Italy = 5,
    #[strum(serialize = "Russia", props(Symbol = "r", Adj = "Russian"))]
    Russia = 6,
    #[strum(serialize = "Turkey", props(Symbol = "t", Adj = "Turkish"))]
    Turkey = 7,
}

/// 国のロジック
impl Power {
    #[cfg(test)]
    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    #[cfg(test)]
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        Self::all().find(|p| p.symbol().eq_ignore_ascii_case(symbol))
    }

    pub fn symbol(&self) -> &'static str {
        self.get_str("Symbol").unwrap_or_default()
    }

    pub fn adjective(&self) -> &'static str {
        self.get_str("Adj").unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn name(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_symbol() {
        assert_eq!(Power::from_symbol("a"), Some(Power::Austria));
        assert_eq!(Power::from_symbol("E"), Some(Power::England));
        assert_eq!(Power::from_symbol("x"), None);
    }
}
