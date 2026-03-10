use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;
use strum::EnumProperty;
use strum::EnumString;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString, EnumProperty)]
#[repr(i64)]
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

impl Power {
    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    pub fn symbol(&self) -> &'static str {
        self.get_str("Symbol").unwrap_or_default()
    }

    pub fn adjective(&self) -> &'static str {
        self.get_str("Adj").unwrap_or_default()
    }

    pub fn name(&self) -> String {
        self.to_string()
    }
}

impl From<i64> for Power {
    fn from(id: i64) -> Self {
        Self::all().find(|p| *p as i64 == id).unwrap_or_else(|| panic!("Unknown PowerId: {}", id))
    }
}
