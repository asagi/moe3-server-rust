use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString)]
#[repr(i64)]
pub enum Power {
    #[strum(serialize = "Austria")]
    Austria = 1,
    #[strum(serialize = "England")]
    England = 2,
    #[strum(serialize = "France")]
    France = 3,
    #[strum(serialize = "Germany")]
    Germany = 4,
    #[strum(serialize = "Italy")]
    Italy = 5,
    #[strum(serialize = "Russia")]
    Russia = 6,
    #[strum(serialize = "Turkey")]
    Turkey = 7,
}

impl Power {
    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Austria => "a",
            Self::England => "e",
            Self::France => "f",
            Self::Germany => "g",
            Self::Italy => "i",
            Self::Russia => "r",
            Self::Turkey => "t",
        }
    }

    pub fn name(&self) -> String {
        self.to_string()
    }

    pub fn adjective(&self) -> &'static str {
        match self {
            Self::Austria => "Austrian",
            Self::England => "English",
            Self::France => "French",
            Self::Germany => "German",
            Self::Italy => "Italian",
            Self::Russia => "Russian",
            Self::Turkey => "Turkish",
        }
    }
}

impl From<i64> for Power {
    fn from(id: i64) -> Self {
        Self::all().find(|p| *p as i64 == id).unwrap_or_else(|| panic!("Unknown PowerId: {}", id))
    }
}
