use super::power::Power;
use serde::Deserialize;
use serde::Serialize;
use strum::AsRefStr;
use strum::Display;
use strum::EnumIter;
use strum::EnumProperty;
use strum::EnumString;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString, AsRefStr, EnumProperty)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(i64)]
pub enum Province {
    #[strum(props(Full = "Adriatic Water", Jname = "アドリア海", Terrain = "Water"))]
    Adr = 1,
    #[strum(props(Full = "Aegean Water", Jname = "エーゲ海", Terrain = "Water"))]
    Aeg = 2,
    #[strum(props(Full = "Albania", Jname = "アルバニア", Terrain = "Coast"))]
    Alb = 3,
    #[strum(props(Full = "Ankara", Jname = "アンカラ", Terrain = "Coast", Supply = "true", Home = "Turkey"))]
    Ank = 4,
    #[strum(props(Full = "Apulia", Jname = "アプリア", Terrain = "Coast", Home = "Italy"))]
    Apu = 5,
    #[strum(props(Full = "Armenia", Jname = "アルメニア", Terrain = "Coast", Home = "Turkey"))]
    Arm = 6,
    #[strum(props(Full = "Baltic Water", Jname = "バルト海", Terrain = "Water"))]
    Bal = 7,
    #[strum(props(Full = "Barents Water", Jname = "バレンツ海", Terrain = "Water"))]
    Bar = 8,
    #[strum(props(Full = "Belgium", Jname = "ベルギー", Terrain = "Coast", Supply = "true"))]
    Bel = 9,
    #[strum(props(Full = "Berlin", Jname = "ベルリン", Terrain = "Coast", Supply = "true", Home = "Germany"))]
    Ber = 10,
    #[strum(props(Full = "Black Water", Jname = "黒海", Terrain = "Water"))]
    Bla = 11,
    #[strum(props(Full = "Bohemia", Jname = "ボヘミア", Terrain = "Inland", Home = "Austria"))]
    Boh = 12,
    #[strum(props(Full = "Gulf of Bothnia", Jname = "ボスニア湾", Terrain = "Water"))]
    Bot = 13,
    #[strum(props(Full = "Brest", Jname = "ブレスト", Terrain = "Coast", Supply = "true", Home = "France"))]
    Bre = 14,
    #[strum(props(Full = "Budapest", Jname = "ブダペスト", Terrain = "Inland", Supply = "true", Home = "Austria"))]
    Bud = 15,
    #[strum(props(Full = "Bulgaria", Jname = "ブルガリア", Terrain = "Coast", Supply = "true"))]
    Bul = 16,
    #[strum(props(Full = "Bulgaria(EC)", Jname = "ブルガリア(EC)", Terrain = "Coast"))]
    BulEc = 17,
    #[strum(props(Full = "Bulgaria(SC)", Jname = "ブルガリア(SC)", Terrain = "Coast"))]
    BulSc = 18,
    #[strum(props(Full = "Burgundy", Jname = "ブルゴーニュ", Terrain = "Inland", Home = "France"))]
    Bur = 19,
    #[strum(props(Full = "Clyde", Jname = "クライド", Terrain = "Coast", Home = "England"))]
    Cly = 20,
    #[strum(props(Full = "Constantinople", Jname = "コンスタンティノープル", Terrain = "Coast", Supply = "true", Home = "Turkey"))]
    Con = 21,
    #[strum(props(Full = "Denmark", Jname = "デンマーク", Terrain = "Coast", Supply = "true"))]
    Den = 22,
    #[strum(props(Full = "Eastern Mediterranean", Jname = "東地中海", Terrain = "Water"))]
    Eas = 23,
    #[strum(props(Full = "Edinburgh", Jname = "エディンバラ", Terrain = "Coast", Supply = "true", Home = "England"))]
    Edi = 24,
    #[strum(props(Full = "English Channel", Jname = "イギリス海峡", Terrain = "Water"))]
    Eng = 25,
    #[strum(props(Full = "Finland", Jname = "フィンランド", Terrain = "Coast", Home = "Russia"))]
    Fin = 26,
    #[strum(props(Full = "Galicia", Jname = "ガリツィア", Terrain = "Inland", Home = "Austria"))]
    Gal = 27,
    #[strum(props(Full = "Gascony", Jname = "ガスコーニュ", Terrain = "Coast", Home = "France"))]
    Gas = 28,
    #[strum(props(Full = "Greece", Jname = "ギリシア", Terrain = "Coast", Supply = "true"))]
    Gre = 29,
    #[strum(props(Full = "Helgoland Bight", Jname = "ヘルゴラント湾", Terrain = "Water"))]
    Hel = 30,
    #[strum(props(Full = "Holland", Jname = "オランダ", Terrain = "Coast", Supply = "true"))]
    Hol = 31,
    #[strum(props(Full = "Ionian Water", Jname = "イオニア海", Terrain = "Water"))]
    Ion = 32,
    #[strum(props(Full = "Irish Water", Jname = "アイリッシュ海", Terrain = "Water"))]
    Iri = 33,
    #[strum(props(Full = "Kiel", Jname = "キール", Terrain = "Coast", Supply = "true", Home = "Germany"))]
    Kie = 34,
    #[strum(props(Full = "London", Jname = "ロンドン", Terrain = "Coast", Supply = "true", Home = "England"))]
    Lon = 35,
    #[strum(props(Full = "Livonia", Jname = "リヴォニア", Terrain = "Coast", Home = "Russia"))]
    Lvn = 36,
    #[strum(props(Full = "Liverpool", Jname = "リヴァプール", Terrain = "Coast", Supply = "true", Home = "England"))]
    Lvp = 37,
    #[strum(props(Full = "Gulf of Lyon", Jname = "リオン湾", Terrain = "Water"))]
    Lyo = 38,
    #[strum(props(Full = "Mid-Atlantic Ocean", Jname = "中大西洋", Terrain = "Water"))]
    Mao = 39,
    #[strum(props(Full = "Marseilles", Jname = "マルセイユ", Terrain = "Coast", Supply = "true", Home = "France"))]
    Mar = 40,
    #[strum(props(Full = "Moscow", Jname = "モスクワ", Terrain = "Inland", Supply = "true", Home = "Russia"))]
    Mos = 41,
    #[strum(props(Full = "Munich", Jname = "ミュンヘン", Terrain = "Inland", Supply = "true", Home = "Germany"))]
    Mun = 42,
    #[strum(props(Full = "North Africa", Jname = "北アフリカ", Terrain = "Coast"))]
    Naf = 43,
    #[strum(props(Full = "North Atlantic Ocean", Jname = "北大西洋", Terrain = "Water"))]
    Nao = 44,
    #[strum(props(Full = "Naples", Jname = "ナポリ", Terrain = "Coast", Supply = "true", Home = "Italy"))]
    Nap = 45,
    #[strum(props(Full = "North Water", Jname = "北海", Terrain = "Water"))]
    Nth = 46,
    #[strum(props(Full = "Norwegian Water", Jname = "ノルウェー海", Terrain = "Water"))]
    Nwg = 47,
    #[strum(props(Full = "Norway", Jname = "ノルウェー", Terrain = "Coast", Supply = "true"))]
    Nwy = 48,
    #[strum(props(Full = "Paris", Jname = "パリ", Terrain = "Inland", Supply = "true", Home = "France"))]
    Par = 49,
    #[strum(props(Full = "Picardy", Jname = "ピカルディ", Terrain = "Coast", Home = "France"))]
    Pic = 50,
    #[strum(props(Full = "Piedmont", Jname = "ピエモンテ", Terrain = "Coast", Home = "Italy"))]
    Pie = 51,
    #[strum(props(Full = "Portugal", Jname = "ポルトガル", Terrain = "Coast", Supply = "true"))]
    Por = 52,
    #[strum(props(Full = "Prussia", Jname = "プロイセン", Terrain = "Coast", Home = "Germany"))]
    Pru = 53,
    #[strum(props(Full = "Rome", Jname = "ローマ", Terrain = "Coast", Supply = "true", Home = "Italy"))]
    Rom = 54,
    #[strum(props(Full = "Ruhr", Jname = "ルール", Terrain = "Inland", Home = "Germany"))]
    Ruh = 55,
    #[strum(props(Full = "Rumania", Jname = "ルーマニア", Terrain = "Coast", Supply = "true"))]
    Rum = 56,
    #[strum(props(Full = "Serbia", Jname = "セルビア", Terrain = "Inland", Supply = "true"))]
    Ser = 57,
    #[strum(props(Full = "Sevastopol", Jname = "セヴァストポリ", Terrain = "Coast", Supply = "true", Home = "Russia"))]
    Sev = 58,
    #[strum(props(Full = "Silesia", Jname = "シレジア", Terrain = "Inland", Home = "Germany"))]
    Sil = 59,
    #[strum(props(Full = "Skagerrak", Jname = "スカゲラク海峡", Terrain = "Water"))]
    Ska = 60,
    #[strum(props(Full = "Smyrna", Jname = "スミルナ", Terrain = "Coast", Supply = "true", Home = "Turkey"))]
    Smy = 61,
    #[strum(props(Full = "Spain", Jname = "スペイン", Terrain = "Coast", Supply = "true"))]
    Spa = 62,
    #[strum(props(Full = "Spain(NC)", Jname = "スペイン(NC)", Terrain = "Coast"))]
    SpaNc = 63,
    #[strum(props(Full = "Spain(SC)", Jname = "スペイン(SC)", Terrain = "Coast"))]
    SpaSc = 64,
    #[strum(props(Full = "St. Petersburg", Jname = "サンクトペテルブルク", Terrain = "Coast", Supply = "true", Home = "Russia"))]
    Stp = 65,
    #[strum(props(Full = "St. Petersburg(NC)", Jname = "サンクトペテルブルク(NC)", Terrain = "Coast"))]
    StpNc = 66,
    #[strum(props(Full = "St. Petersburg(SC)", Jname = "サンクトペテルブルク(SC)", Terrain = "Coast"))]
    StpSc = 67,
    #[strum(props(Full = "Sweden", Jname = "スウェーデン", Terrain = "Coast", Supply = "true"))]
    Swe = 68,
    #[strum(props(Full = "Syria", Jname = "シリア", Terrain = "Coast", Home = "Turkey"))]
    Syr = 69,
    #[strum(props(Full = "Trieste", Jname = "トリエステ", Terrain = "Coast", Supply = "true", Home = "Austria"))]
    Tri = 70,
    #[strum(props(Full = "Tunis", Jname = "チュニス", Terrain = "Coast", Supply = "true"))]
    Tun = 71,
    #[strum(props(Full = "Tuscany", Jname = "トスカーナ", Terrain = "Coast", Home = "Italy"))]
    Tus = 72,
    #[strum(props(Full = "Tyrolia", Jname = "ティロル", Terrain = "Inland", Home = "Austria"))]
    Tyr = 73,
    #[strum(props(Full = "Tyrrhenian Water", Jname = "ティレニア海", Terrain = "Water"))]
    Tys = 74,
    #[strum(props(Full = "Ukraine", Jname = "ウクライナ", Terrain = "Inland", Home = "Russia"))]
    Ukr = 75,
    #[strum(props(Full = "Venice", Jname = "ヴェネツィア", Terrain = "Coast", Supply = "true", Home = "Italy"))]
    Ven = 76,
    #[strum(props(Full = "Vienna", Jname = "ウィーン", Terrain = "Inland", Supply = "true", Home = "Austria"))]
    Vie = 77,
    #[strum(props(Full = "Wales", Jname = "ウェールズ", Terrain = "Coast", Home = "England"))]
    Wal = 78,
    #[strum(props(Full = "Warsaw", Jname = "ワルシャワ", Terrain = "Inland", Supply = "true", Home = "Russia"))]
    War = 79,
    #[strum(props(Full = "Western Mediterranean", Jname = "西地中海", Terrain = "Water"))]
    Wes = 80,
    #[strum(props(Full = "Yorkshire", Jname = "ヨークシャー", Terrain = "Coast", Home = "England"))]
    Yor = 81,
}

impl Province {
    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    pub fn fullname(&self) -> String {
        self.get_str("Full").map(|s| s.to_string()).unwrap_or_else(|| self.to_string())
    }

    pub fn jname(&self) -> &str {
        self.get_str("Jname").unwrap_or("不明")
    }

    pub fn is_suppliable(&self) -> bool {
        self.get_str("Supply") == Some("true")
    }

    pub fn terrain_str(&self) -> &str {
        self.get_str("Terrain").unwrap_or("Unknown")
    }

    pub fn is_water(&self) -> bool {
        self.terrain_str() == "Water"
    }

    pub fn is_coast(&self) -> bool {
        self.terrain_str() == "Coast"
    }

    pub fn is_inland(&self) -> bool {
        self.terrain_str() == "Inland"
    }

    pub fn home_power(&self) -> Option<Power> {
        self.get_str("Home").and_then(|s| s.parse::<Power>().ok())
    }
}

impl From<i64> for Province {
    fn from(id: i64) -> Self {
        Self::all().find(|p| *p as i64 == id).unwrap_or_else(|| panic!("Unknown Province ID: {}", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_province_props() {
        let p = Province::Par;
        assert_eq!(p.jname(), "パリ");
        assert_eq!(p.fullname(), "Paris");
        assert!(p.is_inland());
        assert!(p.is_suppliable());
        assert_eq!(p.home_power(), Some(Power::France));

        let water = Province::Adr;
        assert!(water.is_water());
        assert!(!water.is_suppliable());
    }

    #[test]
    fn test_province_serialization() {
        let p = Province::Adr;
        let serialized = serde_json::to_string(&p).unwrap();
        assert_eq!(serialized, "\"adr\"");

        let p_ec = Province::BulEc;
        let serialized_ec = serde_json::to_string(&p_ec).unwrap();
        assert_eq!(serialized_ec, "\"bul_ec\"");
    }
}
