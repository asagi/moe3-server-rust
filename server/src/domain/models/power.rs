// external crates
use strum::Display;
use strum::EnumIter;
use strum::EnumProperty;
use strum::EnumString;
use strum::IntoEnumIterator;

/// 国の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, EnumString, EnumProperty)]
pub(crate) enum Power {
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
    pub(crate) fn from_symbol(symbol: &str) -> Option<Self> {
        Self::iter().find(|p| p.symbol().eq_ignore_ascii_case(symbol))
    }

    pub(crate) fn symbol(&self) -> &'static str {
        self.get_str("Symbol").unwrap_or_default()
    }

    pub(crate) fn adjective(&self) -> &'static str {
        self.get_str("Adj").unwrap_or_default()
    }

    #[allow(dead_code)]
    pub(crate) fn name(&self) -> String {
        self.to_string()
    }
}

impl serde::Serialize for Power {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.symbol())
    }
}

impl<'de> serde::Deserialize<'de> for Power {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_symbol(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid power code: {}", s)))
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
