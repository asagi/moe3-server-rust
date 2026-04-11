use super::power::Power;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Province(u8);

#[derive(Debug, Clone, Copy)]
pub struct ProvinceData {
    pub code: &'static str,
    pub full: &'static str,
    pub jname: &'static str,
    pub kind: &'static str,
    pub supply: bool,
    pub home: Option<&'static str>,
}

#[rustfmt::skip]
const PROVINCE_DATA: &[ProvinceData] = &[
    ProvinceData { code: "adr", full: "Adriatic Water", jname: "アドリア海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "aeg", full: "Aegean Water", jname: "エーゲ海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "alb", full: "Albania", jname: "アルバニア", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "ank", full: "Ankara", jname: "アンカラ", kind: "Coast", supply: true, home: Some("t") },
    ProvinceData { code: "apu", full: "Apulia", jname: "アプリア", kind: "Coast", supply: false, home: Some("i") },
    ProvinceData { code: "arm", full: "Armenia", jname: "アルメニア", kind: "Coast", supply: false, home: Some("t") },
    ProvinceData { code: "bal", full: "Baltic Water", jname: "バルト海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "bar", full: "Barents Water", jname: "バレンツ海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "bel", full: "Belgium", jname: "ベルギー", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "ber", full: "Berlin", jname: "ベルリン", kind: "Coast", supply: true, home: Some("g") },
    ProvinceData { code: "bla", full: "Black Water", jname: "黒海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "boh", full: "Bohemia", jname: "ボヘミア", kind: "Inland", supply: false, home: Some("a") },
    ProvinceData { code: "bot", full: "Gulf of Bothnia", jname: "ボスニア湾", kind: "Water", supply: false, home: None },
    ProvinceData { code: "bre", full: "Brest", jname: "ブレスト", kind: "Coast", supply: true, home: Some("f") },
    ProvinceData { code: "bud", full: "Budapest", jname: "ブダペスト", kind: "Inland", supply: true, home: Some("a") },
    ProvinceData { code: "bul", full: "Bulgaria", jname: "ブルガリア", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "bul_ec", full: "Bulgaria(EC)", jname: "ブルガリア(EC)", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "bul_sc", full: "Bulgaria(SC)", jname: "ブルガリア(SC)", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "bur", full: "Burgundy", jname: "ブルゴーニュ", kind: "Inland", supply: false, home: Some("f") },
    ProvinceData { code: "cly", full: "Clyde", jname: "クライド", kind: "Coast", supply: false, home: Some("e") },
    ProvinceData { code: "con", full: "Constantinople", jname: "コンスタンティノープル", kind: "Coast", supply: true, home: Some("t") },
    ProvinceData { code: "den", full: "Denmark", jname: "デンマーク", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "eas", full: "Eastern Mediterranean", jname: "東地中海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "edi", full: "Edinburgh", jname: "エディンバラ", kind: "Coast", supply: true, home: Some("e") },
    ProvinceData { code: "eng", full: "English Channel", jname: "イギリス海峡", kind: "Water", supply: false, home: None },
    ProvinceData { code: "fin", full: "Finland", jname: "フィンランド", kind: "Coast", supply: false, home: Some("r") },
    ProvinceData { code: "gal", full: "Galicia", jname: "ガリツィア", kind: "Inland", supply: false, home: Some("a") },
    ProvinceData { code: "gas", full: "Gascony", jname: "ガスコーニュ", kind: "Coast", supply: false, home: Some("f") },
    ProvinceData { code: "gre", full: "Greece", jname: "ギリシア", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "hel", full: "Helgoland Bight", jname: "ヘルゴラント湾", kind: "Water", supply: false, home: None },
    ProvinceData { code: "hol", full: "Holland", jname: "オランダ", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "ion", full: "Ionian Water", jname: "イオニア海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "iri", full: "Irish Water", jname: "アイリッシュ海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "kie", full: "Kiel", jname: "キール", kind: "Coast", supply: true, home: Some("g") },
    ProvinceData { code: "lon", full: "London", jname: "ロンドン", kind: "Coast", supply: true, home: Some("e") },
    ProvinceData { code: "lvn", full: "Livonia", jname: "リヴォニア", kind: "Coast", supply: false, home: Some("r") },
    ProvinceData { code: "lvp", full: "Liverpool", jname: "リヴァプール", kind: "Coast", supply: true, home: Some("e") },
    ProvinceData { code: "lyo", full: "Gulf of Lyon", jname: "リオン湾", kind: "Water", supply: false, home: None },
    ProvinceData { code: "mao", full: "Mid-Atlantic Ocean", jname: "中大西洋", kind: "Water", supply: false, home: None },
    ProvinceData { code: "mar", full: "Marseilles", jname: "マルセイユ", kind: "Coast", supply: true, home: Some("f") },
    ProvinceData { code: "mos", full: "Moscow", jname: "モスクワ", kind: "Inland", supply: true, home: Some("r") },
    ProvinceData { code: "mun", full: "Munich", jname: "ミュンヘン", kind: "Inland", supply: true, home: Some("g") },
    ProvinceData { code: "naf", full: "North Africa", jname: "北アフリカ", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "nao", full: "North Atlantic Ocean", jname: "北大西洋", kind: "Water", supply: false, home: None },
    ProvinceData { code: "nap", full: "Naples", jname: "ナポリ", kind: "Coast", supply: true, home: Some("i") },
    ProvinceData { code: "nth", full: "North Water", jname: "北海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "nwg", full: "Norwegian Water", jname: "ノルウェー海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "nwy", full: "Norway", jname: "ノルウェー", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "par", full: "Paris", jname: "パリ", kind: "Inland", supply: true, home: Some("f") },
    ProvinceData { code: "pic", full: "Picardy", jname: "ピカルディ", kind: "Coast", supply: false, home: Some("f") },
    ProvinceData { code: "pie", full: "Piedmont", jname: "ピエモンテ", kind: "Coast", supply: false, home: Some("i") },
    ProvinceData { code: "por", full: "Portugal", jname: "ポルトガル", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "pru", full: "Prussia", jname: "プロイセン", kind: "Coast", supply: false, home: Some("g") },
    ProvinceData { code: "rom", full: "Rome", jname: "ローマ", kind: "Coast", supply: true, home: Some("i") },
    ProvinceData { code: "ruh", full: "Ruhr", jname: "ルール", kind: "Inland", supply: false, home: Some("g") },
    ProvinceData { code: "rum", full: "Rumania", jname: "ルーマニア", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "ser", full: "Serbia", jname: "セルビア", kind: "Inland", supply: true, home: None },
    ProvinceData { code: "sev", full: "Sevastopol", jname: "セヴァストポリ", kind: "Coast", supply: true, home: Some("r") },
    ProvinceData { code: "sil", full: "Silesia", jname: "シレジア", kind: "Inland", supply: false, home: Some("g") },
    ProvinceData { code: "ska", full: "Skagerrak", jname: "スカゲラク海峡", kind: "Water", supply: false, home: None },
    ProvinceData { code: "smy", full: "Smyrna", jname: "スミルナ", kind: "Coast", supply: true, home: Some("t") },
    ProvinceData { code: "spa", full: "Spain", jname: "スペイン", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "spa_nc", full: "Spain(NC)", jname: "スペイン(NC)", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "spa_sc", full: "Spain(SC)", jname: "スペイン(SC)", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "stp", full: "St. Petersburg", jname: "サンクトペテルブルク", kind: "Coast", supply: true, home: Some("r") },
    ProvinceData { code: "stp_nc", full: "St. Petersburg(NC)", jname: "サンクトペテルブルク(NC)", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "stp_sc", full: "St. Petersburg(SC)", jname: "サンクトペテルブルク(SC)", kind: "Coast", supply: false, home: None },
    ProvinceData { code: "swe", full: "Sweden", jname: "スウェーデン", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "syr", full: "Syria", jname: "シリア", kind: "Coast", supply: false, home: Some("t") },
    ProvinceData { code: "tri", full: "Trieste", jname: "トリエステ", kind: "Coast", supply: true, home: Some("a") },
    ProvinceData { code: "tun", full: "Tunis", jname: "チュニス", kind: "Coast", supply: true, home: None },
    ProvinceData { code: "tus", full: "Tuscany", jname: "トスカーナ", kind: "Coast", supply: false, home: Some("i") },
    ProvinceData { code: "tyr", full: "Tyrolia", jname: "ティロル", kind: "Inland", supply: false, home: Some("a") },
    ProvinceData { code: "tys", full: "Tyrrhenian Water", jname: "ティレニア海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "ukr", full: "Ukraine", jname: "ウクライナ", kind: "Inland", supply: false, home: Some("r") },
    ProvinceData { code: "ven", full: "Venice", jname: "ヴェネツィア", kind: "Coast", supply: true, home: Some("i") },
    ProvinceData { code: "vie", full: "Vienna", jname: "ウィーン", kind: "Inland", supply: true, home: Some("a") },
    ProvinceData { code: "wal", full: "Wales", jname: "ウェールズ", kind: "Coast", supply: false, home: Some("e") },
    ProvinceData { code: "war", full: "Warsaw", jname: "ワルシャワ", kind: "Inland", supply: true, home: Some("r") },
    ProvinceData { code: "wes", full: "Western Mediterranean", jname: "西地中海", kind: "Water", supply: false, home: None },
    ProvinceData { code: "yor", full: "Yorkshire", jname: "ヨークシャー", kind: "Coast", supply: false, home: Some("e") },
];

impl Province {
    pub fn from_code(code: &str) -> Option<Self> {
        PROVINCE_DATA.iter().position(|d| d.code == code).map(|idx| Self(idx as u8))
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (0..PROVINCE_DATA.len()).map(|idx| Self(idx as u8))
    }

    pub fn data(self) -> &'static ProvinceData {
        &PROVINCE_DATA[self.0 as usize]
    }

    pub fn full_name(self) -> &'static str {
        self.data().full
    }

    pub fn jname(self) -> &'static str {
        self.data().jname
    }

    pub fn kind(self) -> &'static str {
        self.data().kind
    }

    pub fn is_supply_center(self) -> bool {
        self.data().supply
    }

    pub fn home(self) -> Option<&'static str> {
        self.data().home
    }

    pub fn is_water(self) -> bool {
        self.kind() == "Water"
    }

    pub fn is_coast(self) -> bool {
        self.kind() == "Coast"
    }

    pub fn is_inland(self) -> bool {
        self.kind() == "Inland"
    }

    pub fn code(self) -> &'static str {
        self.data().code
    }

    /// 指定コードが指定国の初期補給都市であるかを返却する
    pub fn is_home_sc(code: &str, power: &Power) -> bool {
        PROVINCE_DATA
            .iter()
            .any(|d| d.code == code && d.supply && d.home == Some(power.symbol()))
    }
}

impl fmt::Display for Province {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl TryFrom<String> for Province {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_code(&value).ok_or_else(|| format!("invalid province code: {value}"))
    }
}

impl From<Province> for String {
    fn from(value: Province) -> Self {
        value.code().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(code: &str) -> Province {
        Province::from_code(code).expect("valid province code")
    }

    #[test]
    fn test_from_code() {
        assert_eq!(Province::from_code("adr"), Some(p("adr")));
        assert_eq!(Province::from_code("lon"), Some(p("lon")));
        assert_eq!(Province::from_code("invalid"), None);
    }

    #[test]
    fn test_full_name() {
        assert_eq!(p("adr").full_name(), "Adriatic Water");
        assert_eq!(p("lon").full_name(), "London");
    }

    #[test]
    fn test_is_supply_center() {
        assert!(p("lon").is_supply_center());
        assert!(!p("adr").is_supply_center());
    }

    #[test]
    fn test_is_water() {
        assert!(p("adr").is_water());
        assert!(!p("lon").is_water());
    }

    #[test]
    fn test_home() {
        assert_eq!(p("lon").home(), Some("e"));
        assert_eq!(p("adr").home(), None);
    }
}
