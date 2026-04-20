#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// standard library
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;

// structs
use super::Path;
use super::Power;

// ============================================================================
// definitions
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Province(&'static str);

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProvinceData {
    pub(crate) code: &'static str,
    pub(crate) short: &'static str,
    pub(crate) full: &'static str,
    pub(crate) jname: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) supply: bool,
    pub(crate) home: Option<&'static str>,
}

#[rustfmt::skip]
const PROVINCE_DATA: &[ProvinceData] = &[
    ProvinceData { code: "adr",    short: "Adr",     full: "Adriatic Sea",          jname: "アドリア海",               kind: "Water",  supply: false, home: None },
    ProvinceData { code: "aeg",    short: "Aeg",     full: "Aegean Sea",            jname: "エーゲ海",                 kind: "Water",  supply: false, home: None },
    ProvinceData { code: "alb",    short: "Alb",     full: "Albania",               jname: "アルバニア",               kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "ank",    short: "Ank",     full: "Ankara",                jname: "アンカラ",                 kind: "Coast",  supply: true,  home: Some("t") },
    ProvinceData { code: "apu",    short: "Apu",     full: "Apulia",                jname: "アプリア",                 kind: "Coast",  supply: false, home: Some("i") },
    ProvinceData { code: "arm",    short: "Arm",     full: "Armenia",               jname: "アルメニア",               kind: "Coast",  supply: false, home: Some("t") },
    ProvinceData { code: "bal",    short: "Bal",     full: "Baltic Sea",            jname: "バルト海",                 kind: "Water",  supply: false, home: None },
    ProvinceData { code: "bar",    short: "Bar",     full: "Barents Sea",           jname: "バレンツ海",               kind: "Water",  supply: false, home: None },
    ProvinceData { code: "bel",    short: "Bel",     full: "Belgium",               jname: "ベルギー",                 kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "ber",    short: "Ber",     full: "Berlin",                jname: "ベルリン",                 kind: "Coast",  supply: true,  home: Some("g") },
    ProvinceData { code: "bla",    short: "Bla",     full: "Black Sea",             jname: "黒海",                     kind: "Water",  supply: false, home: None },
    ProvinceData { code: "boh",    short: "Boh",     full: "Bohemia",               jname: "ボヘミア",                 kind: "Inland", supply: false, home: Some("a") },
    ProvinceData { code: "bot",    short: "Bot",     full: "Gulf of Bothnia",       jname: "ボスニア湾",               kind: "Water",  supply: false, home: None },
    ProvinceData { code: "bre",    short: "Bre",     full: "Brest",                 jname: "ブレスト",                 kind: "Coast",  supply: true,  home: Some("f") },
    ProvinceData { code: "bud",    short: "Bud",     full: "Budapest",              jname: "ブダペスト",               kind: "Inland", supply: true,  home: Some("a") },
    ProvinceData { code: "bul",    short: "Bul",     full: "Bulgaria",              jname: "ブルガリア",               kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "bul_ec", short: "Bul(EC)", full: "Bulgaria(EC)",          jname: "ブルガリア(EC)",           kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "bul_sc", short: "Bul(SC)", full: "Bulgaria(SC)",          jname: "ブルガリア(SC)",           kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "bur",    short: "Bur",     full: "Burgundy",              jname: "ブルゴーニュ",             kind: "Inland", supply: false, home: Some("f") },
    ProvinceData { code: "cly",    short: "Cly",     full: "Clyde",                 jname: "クライド",                 kind: "Coast",  supply: false, home: Some("e") },
    ProvinceData { code: "con",    short: "Con",     full: "Constantinople",        jname: "コンスタンティノープル",   kind: "Coast",  supply: true,  home: Some("t") },
    ProvinceData { code: "den",    short: "Den",     full: "Denmark",               jname: "デンマーク",               kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "eas",    short: "Eas",     full: "Eastern Mediterranean", jname: "東地中海",                 kind: "Water",  supply: false, home: None },
    ProvinceData { code: "edi",    short: "Edi",     full: "Edinburgh",             jname: "エディンバラ",             kind: "Coast",  supply: true,  home: Some("e") },
    ProvinceData { code: "eng",    short: "Eng",     full: "English Channel",       jname: "イギリス海峡",             kind: "Water",  supply: false, home: None },
    ProvinceData { code: "fin",    short: "Fin",     full: "Finland",               jname: "フィンランド",             kind: "Coast",  supply: false, home: Some("r") },
    ProvinceData { code: "gal",    short: "Gal",     full: "Galicia",               jname: "ガリツィア",               kind: "Inland", supply: false, home: Some("a") },
    ProvinceData { code: "gas",    short: "Gas",     full: "Gascony",               jname: "ガスコーニュ",             kind: "Coast",  supply: false, home: Some("f") },
    ProvinceData { code: "gre",    short: "Gre",     full: "Greece",                jname: "ギリシア",                 kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "hel",    short: "Hel",     full: "Helgoland Bight",       jname: "ヘルゴラント湾",           kind: "Water",  supply: false, home: None },
    ProvinceData { code: "hol",    short: "Hol",     full: "Holland",               jname: "オランダ",                 kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "ion",    short: "Ion",     full: "Ionian Sea",            jname: "イオニア海",               kind: "Water",  supply: false, home: None },
    ProvinceData { code: "iri",    short: "Iri",     full: "Irish Sea",             jname: "アイリッシュ海",           kind: "Water",  supply: false, home: None },
    ProvinceData { code: "kie",    short: "Kie",     full: "Kiel",                  jname: "キール",                   kind: "Coast",  supply: true,  home: Some("g") },
    ProvinceData { code: "lon",    short: "Lon",     full: "London",                jname: "ロンドン",                 kind: "Coast",  supply: true,  home: Some("e") },
    ProvinceData { code: "lvn",    short: "Lvn",     full: "Livonia",               jname: "リヴォニア",               kind: "Coast",  supply: false, home: Some("r") },
    ProvinceData { code: "lvp",    short: "Lvp",     full: "Liverpool",             jname: "リヴァプール",             kind: "Coast",  supply: true,  home: Some("e") },
    ProvinceData { code: "gol",    short: "GoL",     full: "Gulf of Lyon",          jname: "リオン湾",                 kind: "Water",  supply: false, home: None },
    ProvinceData { code: "mid",    short: "Mid",     full: "Mid-Atlantic Ocean",    jname: "中大西洋",                 kind: "Water",  supply: false, home: None },
    ProvinceData { code: "mar",    short: "Mar",     full: "Marseilles",            jname: "マルセイユ",               kind: "Coast",  supply: true,  home: Some("f") },
    ProvinceData { code: "mos",    short: "Mos",     full: "Moscow",                jname: "モスクワ",                 kind: "Inland", supply: true,  home: Some("r") },
    ProvinceData { code: "mun",    short: "Mun",     full: "Munich",                jname: "ミュンヘン",               kind: "Inland", supply: true,  home: Some("g") },
    ProvinceData { code: "naf",    short: "NAf",     full: "North Africa",          jname: "北アフリカ",               kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "nat",    short: "NAt",     full: "North Atlantic Ocean",  jname: "北大西洋",                 kind: "Water",  supply: false, home: None },
    ProvinceData { code: "nap",    short: "Nap",     full: "Naples",                jname: "ナポリ",                   kind: "Coast",  supply: true,  home: Some("i") },
    ProvinceData { code: "nth",    short: "Nth",     full: "North Sea",             jname: "北海",                     kind: "Water",  supply: false, home: None },
    ProvinceData { code: "nrg",    short: "Nrg",     full: "Norwegian Sea",         jname: "ノルウェー海",             kind: "Water",  supply: false, home: None },
    ProvinceData { code: "nwy",    short: "Nwy",     full: "Norway",                jname: "ノルウェー",               kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "par",    short: "Par",     full: "Paris",                 jname: "パリ",                     kind: "Inland", supply: true,  home: Some("f") },
    ProvinceData { code: "pic",    short: "Pic",     full: "Picardy",               jname: "ピカルディ",               kind: "Coast",  supply: false, home: Some("f") },
    ProvinceData { code: "pie",    short: "Pie",     full: "Piedmont",              jname: "ピエモンテ",               kind: "Coast",  supply: false, home: Some("i") },
    ProvinceData { code: "por",    short: "Por",     full: "Portugal",              jname: "ポルトガル",               kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "pru",    short: "Pru",     full: "Prussia",               jname: "プロイセン",               kind: "Coast",  supply: false, home: Some("g") },
    ProvinceData { code: "rom",    short: "Rom",     full: "Rome",                  jname: "ローマ",                   kind: "Coast",  supply: true,  home: Some("i") },
    ProvinceData { code: "ruh",    short: "Ruh",     full: "Ruhr",                  jname: "ルール",                   kind: "Inland", supply: false, home: Some("g") },
    ProvinceData { code: "rum",    short: "Rum",     full: "Rumania",               jname: "ルーマニア",               kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "ser",    short: "Ser",     full: "Serbia",                jname: "セルビア",                 kind: "Inland", supply: true,  home: None },
    ProvinceData { code: "sev",    short: "Sev",     full: "Sevastopol",            jname: "セヴァストポリ",           kind: "Coast",  supply: true,  home: Some("r") },
    ProvinceData { code: "sil",    short: "Sil",     full: "Silesia",               jname: "シレジア",                 kind: "Inland", supply: false, home: Some("g") },
    ProvinceData { code: "ska",    short: "Ska",     full: "Skagerrak",             jname: "スカゲラク海峡",           kind: "Water",  supply: false, home: None },
    ProvinceData { code: "smy",    short: "Smy",     full: "Smyrna",                jname: "スミルナ",                 kind: "Coast",  supply: true,  home: Some("t") },
    ProvinceData { code: "spa",    short: "Spa",     full: "Spain",                 jname: "スペイン",                 kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "spa_nc", short: "Spa(NC)", full: "Spain(NC)",             jname: "スペイン(NC)",             kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "spa_sc", short: "Spa(SC)", full: "Spain(SC)",             jname: "スペイン(SC)",             kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "stp",    short: "StP",     full: "St. Petersburg",        jname: "サンクトペテルブルク",     kind: "Coast",  supply: true,  home: Some("r") },
    ProvinceData { code: "stp_nc", short: "StP(NC)", full: "St. Petersburg(NC)",    jname: "サンクトペテルブルク(NC)", kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "stp_sc", short: "StP(SC)", full: "St. Petersburg(SC)",    jname: "サンクトペテルブルク(SC)", kind: "Coast",  supply: false, home: None },
    ProvinceData { code: "swe",    short: "Swe",     full: "Sweden",                jname: "スウェーデン",             kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "syr",    short: "Syr",     full: "Syria",                 jname: "シリア",                   kind: "Coast",  supply: false, home: Some("t") },
    ProvinceData { code: "tri",    short: "Tri",     full: "Trieste",               jname: "トリエステ",               kind: "Coast",  supply: true,  home: Some("a") },
    ProvinceData { code: "tun",    short: "Tun",     full: "Tunis",                 jname: "チュニス",                 kind: "Coast",  supply: true,  home: None },
    ProvinceData { code: "tus",    short: "Tus",     full: "Tuscany",               jname: "トスカーナ",               kind: "Coast",  supply: false, home: Some("i") },
    ProvinceData { code: "tyr",    short: "Tyr",     full: "Tyrolia",               jname: "ティロル",                 kind: "Inland", supply: false, home: Some("a") },
    ProvinceData { code: "tyn",    short: "Tyn",     full: "Tyrrhenian Sea",        jname: "ティレニア海",             kind: "Water",  supply: false, home: None },
    ProvinceData { code: "ukr",    short: "Ukr",     full: "Ukraine",               jname: "ウクライナ",               kind: "Inland", supply: false, home: Some("r") },
    ProvinceData { code: "ven",    short: "Ven",     full: "Venice",                jname: "ヴェネツィア",             kind: "Coast",  supply: true,  home: Some("i") },
    ProvinceData { code: "vie",    short: "Vie",     full: "Vienna",                jname: "ウィーン",                 kind: "Inland", supply: true,  home: Some("a") },
    ProvinceData { code: "wal",    short: "Wal",     full: "Wales",                 jname: "ウェールズ",               kind: "Coast",  supply: false, home: Some("e") },
    ProvinceData { code: "war",    short: "War",     full: "Warsaw",                jname: "ワルシャワ",               kind: "Inland", supply: true,  home: Some("r") },
    ProvinceData { code: "wes",    short: "Wes",     full: "Western Mediterranean", jname: "西地中海",                 kind: "Water",  supply: false, home: None },
    ProvinceData { code: "yor",    short: "Yor",     full: "Yorkshire",             jname: "ヨークシャー",             kind: "Coast",  supply: false, home: Some("e") },
];

impl Province {
    pub(crate) fn from_code(code: &str) -> Option<Self> {
        PROVINCE_DATA.iter().find(|d| d.code == code).map(|d| Self(d.code))
    }

    #[allow(dead_code)]
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        (0..PROVINCE_DATA.len()).map(|idx| Self(PROVINCE_DATA[idx].code))
    }

    pub(crate) fn data(self) -> &'static ProvinceData {
        PROVINCE_DATA.iter().find(|d| d.code == self.0).expect("valid province code")
    }

    pub(crate) fn full_name(self) -> &'static str {
        self.data().full
    }

    pub(crate) fn short_name(self) -> &'static str {
        self.data().short
    }

    #[allow(dead_code)]
    pub(crate) fn jname(self) -> &'static str {
        self.data().jname
    }

    pub(crate) fn kind(self) -> &'static str {
        self.data().kind
    }

    pub(crate) fn is_supply_center(self) -> bool {
        self.data().supply
    }

    #[allow(dead_code)]
    pub(crate) fn home(self) -> Option<&'static str> {
        self.data().home
    }

    pub(crate) fn is_water(self) -> bool {
        self.kind() == "Water"
    }

    pub(crate) fn is_coast(self) -> bool {
        self.kind() == "Coast"
    }

    #[allow(dead_code)]
    pub(crate) fn is_inland(self) -> bool {
        self.kind() == "Inland"
    }

    pub(crate) fn code_with_coast(self) -> &'static str {
        self.data().code
    }

    pub(crate) fn code(self) -> &'static str {
        &self.data().code[..3]
    }

    /// 指定コードが指定国の初期補給都市であるかを返却する
    pub(crate) fn is_home_sc(code: &str, power: &Power) -> bool {
        PROVINCE_DATA
            .iter()
            .any(|d| d.code == code && d.supply && d.home == Some(power.symbol()))
    }

    /// 指定された二点の最短距離を返却する
    pub(crate) fn distance(from: &str, to: &str) -> usize {
        let from_base = from.get(..3).expect("valid province code");
        let to_base = to.get(..3).expect("valid province code");

        if from_base == to_base {
            return 0;
        }

        let base_codes = PROVINCE_DATA
            .iter()
            .filter(|d| d.code.len() == 3)
            .map(|d| d.code)
            .collect::<Vec<_>>();

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(from_base);
        queue.push_back((from_base, 0usize));

        while let Some((current, dist)) = queue.pop_front() {
            for &next in &base_codes {
                if visited.contains(next) || current == next {
                    continue;
                }

                let current_variants = PROVINCE_DATA.iter().filter(|d| &d.code[..3] == current).map(|d| d.code);
                let connected = current_variants.clone().any(|origin| Path::is_adjacent(origin, next))
                    || PROVINCE_DATA
                        .iter()
                        .filter(|d| &d.code[..3] == next)
                        .map(|d| d.code)
                        .any(|origin| Path::is_adjacent(origin, current));

                if !connected {
                    continue;
                }

                if next == to_base {
                    return dist + 1;
                }

                visited.insert(next);
                queue.push_back((next, dist + 1));
            }
        }

        unreachable!("province graph is expected to be fully connected")
    }
}

impl fmt::Display for Province {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code_with_coast())
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
        value.code_with_coast().to_string()
    }
}

// ============================================================================
// tests
// ============================================================================

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
        assert_eq!(p("adr").full_name(), "Adriatic Sea");
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

    #[test]
    fn test_distance_ska_to_stp_is_two() {
        assert_eq!(Province::distance("ska", "stp"), 2);
    }

    #[test]
    fn test_distance_ber_to_stp_is_three() {
        assert_eq!(Province::distance("ber", "stp"), 3);
    }
}
