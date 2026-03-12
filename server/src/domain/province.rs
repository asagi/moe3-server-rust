use super::Power;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use strum::AsRefStr;
use strum::Display;
use strum::EnumIter;
use strum::EnumProperty;
use strum::EnumString;
use strum::IntoEnumIterator;

/// 地域の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString, AsRefStr, EnumProperty)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Province {
    #[strum(props(Full = "Adriatic Water", Jname = "アドリア海", Kind = "Water"))]
    Adr = 1,
    #[strum(props(Full = "Aegean Water", Jname = "エーゲ海", Kind = "Water"))]
    Aeg = 2,
    #[strum(props(Full = "Albania", Jname = "アルバニア", Kind = "Coast"))]
    Alb = 3,
    #[strum(props(Full = "Ankara", Jname = "アンカラ", Kind = "Coast", Supply = "true", Home = "Turkey"))]
    Ank = 4,
    #[strum(props(Full = "Apulia", Jname = "アプリア", Kind = "Coast", Home = "Italy"))]
    Apu = 5,
    #[strum(props(Full = "Armenia", Jname = "アルメニア", Kind = "Coast", Home = "Turkey"))]
    Arm = 6,
    #[strum(props(Full = "Baltic Water", Jname = "バルト海", Kind = "Water"))]
    Bal = 7,
    #[strum(props(Full = "Barents Water", Jname = "バレンツ海", Kind = "Water"))]
    Bar = 8,
    #[strum(props(Full = "Belgium", Jname = "ベルギー", Kind = "Coast", Supply = "true"))]
    Bel = 9,
    #[strum(props(Full = "Berlin", Jname = "ベルリン", Kind = "Coast", Supply = "true", Home = "Germany"))]
    Ber = 10,
    #[strum(props(Full = "Black Water", Jname = "黒海", Kind = "Water"))]
    Bla = 11,
    #[strum(props(Full = "Bohemia", Jname = "ボヘミア", Kind = "Inland", Home = "Austria"))]
    Boh = 12,
    #[strum(props(Full = "Gulf of Bothnia", Jname = "ボスニア湾", Kind = "Water"))]
    Bot = 13,
    #[strum(props(Full = "Brest", Jname = "ブレスト", Kind = "Coast", Supply = "true", Home = "France"))]
    Bre = 14,
    #[strum(props(Full = "Budapest", Jname = "ブダペスト", Kind = "Inland", Supply = "true", Home = "Austria"))]
    Bud = 15,
    #[strum(props(Full = "Bulgaria", Jname = "ブルガリア", Kind = "Coast", Supply = "true"))]
    Bul = 16,
    #[strum(props(Full = "Bulgaria(EC)", Jname = "ブルガリア(EC)", Kind = "Coast"))]
    BulEc = 17,
    #[strum(props(Full = "Bulgaria(SC)", Jname = "ブルガリア(SC)", Kind = "Coast"))]
    BulSc = 18,
    #[strum(props(Full = "Burgundy", Jname = "ブルゴーニュ", Kind = "Inland", Home = "France"))]
    Bur = 19,
    #[strum(props(Full = "Clyde", Jname = "クライド", Kind = "Coast", Home = "England"))]
    Cly = 20,
    #[strum(props(Full = "Constantinople", Jname = "コンスタンティノープル", Kind = "Coast", Supply = "true", Home = "Turkey"))]
    Con = 21,
    #[strum(props(Full = "Denmark", Jname = "デンマーク", Kind = "Coast", Supply = "true"))]
    Den = 22,
    #[strum(props(Full = "Eastern Mediterranean", Jname = "東地中海", Kind = "Water"))]
    Eas = 23,
    #[strum(props(Full = "Edinburgh", Jname = "エディンバラ", Kind = "Coast", Supply = "true", Home = "England"))]
    Edi = 24,
    #[strum(props(Full = "English Channel", Jname = "イギリス海峡", Kind = "Water"))]
    Eng = 25,
    #[strum(props(Full = "Finland", Jname = "フィンランド", Kind = "Coast", Home = "Russia"))]
    Fin = 26,
    #[strum(props(Full = "Galicia", Jname = "ガリツィア", Kind = "Inland", Home = "Austria"))]
    Gal = 27,
    #[strum(props(Full = "Gascony", Jname = "ガスコーニュ", Kind = "Coast", Home = "France"))]
    Gas = 28,
    #[strum(props(Full = "Greece", Jname = "ギリシア", Kind = "Coast", Supply = "true"))]
    Gre = 29,
    #[strum(props(Full = "Helgoland Bight", Jname = "ヘルゴラント湾", Kind = "Water"))]
    Hel = 30,
    #[strum(props(Full = "Holland", Jname = "オランダ", Kind = "Coast", Supply = "true"))]
    Hol = 31,
    #[strum(props(Full = "Ionian Water", Jname = "イオニア海", Kind = "Water"))]
    Ion = 32,
    #[strum(props(Full = "Irish Water", Jname = "アイリッシュ海", Kind = "Water"))]
    Iri = 33,
    #[strum(props(Full = "Kiel", Jname = "キール", Kind = "Coast", Supply = "true", Home = "Germany"))]
    Kie = 34,
    #[strum(props(Full = "London", Jname = "ロンドン", Kind = "Coast", Supply = "true", Home = "England"))]
    Lon = 35,
    #[strum(props(Full = "Livonia", Jname = "リヴォニア", Kind = "Coast", Home = "Russia"))]
    Lvn = 36,
    #[strum(props(Full = "Liverpool", Jname = "リヴァプール", Kind = "Coast", Supply = "true", Home = "England"))]
    Lvp = 37,
    #[strum(props(Full = "Gulf of Lyon", Jname = "リオン湾", Kind = "Water"))]
    Lyo = 38,
    #[strum(props(Full = "Mid-Atlantic Ocean", Jname = "中大西洋", Kind = "Water"))]
    Mao = 39,
    #[strum(props(Full = "Marseilles", Jname = "マルセイユ", Kind = "Coast", Supply = "true", Home = "France"))]
    Mar = 40,
    #[strum(props(Full = "Moscow", Jname = "モスクワ", Kind = "Inland", Supply = "true", Home = "Russia"))]
    Mos = 41,
    #[strum(props(Full = "Munich", Jname = "ミュンヘン", Kind = "Inland", Supply = "true", Home = "Germany"))]
    Mun = 42,
    #[strum(props(Full = "North Africa", Jname = "北アフリカ", Kind = "Coast"))]
    Naf = 43,
    #[strum(props(Full = "North Atlantic Ocean", Jname = "北大西洋", Kind = "Water"))]
    Nao = 44,
    #[strum(props(Full = "Naples", Jname = "ナポリ", Kind = "Coast", Supply = "true", Home = "Italy"))]
    Nap = 45,
    #[strum(props(Full = "North Water", Jname = "北海", Kind = "Water"))]
    Nth = 46,
    #[strum(props(Full = "Norwegian Water", Jname = "ノルウェー海", Kind = "Water"))]
    Nwg = 47,
    #[strum(props(Full = "Norway", Jname = "ノルウェー", Kind = "Coast", Supply = "true"))]
    Nwy = 48,
    #[strum(props(Full = "Paris", Jname = "パリ", Kind = "Inland", Supply = "true", Home = "France"))]
    Par = 49,
    #[strum(props(Full = "Picardy", Jname = "ピカルディ", Kind = "Coast", Home = "France"))]
    Pic = 50,
    #[strum(props(Full = "Piedmont", Jname = "ピエモンテ", Kind = "Coast", Home = "Italy"))]
    Pie = 51,
    #[strum(props(Full = "Portugal", Jname = "ポルトガル", Kind = "Coast", Supply = "true"))]
    Por = 52,
    #[strum(props(Full = "Prussia", Jname = "プロイセン", Kind = "Coast", Home = "Germany"))]
    Pru = 53,
    #[strum(props(Full = "Rome", Jname = "ローマ", Kind = "Coast", Supply = "true", Home = "Italy"))]
    Rom = 54,
    #[strum(props(Full = "Ruhr", Jname = "ルール", Kind = "Inland", Home = "Germany"))]
    Ruh = 55,
    #[strum(props(Full = "Rumania", Jname = "ルーマニア", Kind = "Coast", Supply = "true"))]
    Rum = 56,
    #[strum(props(Full = "Serbia", Jname = "セルビア", Kind = "Inland", Supply = "true"))]
    Ser = 57,
    #[strum(props(Full = "Sevastopol", Jname = "セヴァストポリ", Kind = "Coast", Supply = "true", Home = "Russia"))]
    Sev = 58,
    #[strum(props(Full = "Silesia", Jname = "シレジア", Kind = "Inland", Home = "Germany"))]
    Sil = 59,
    #[strum(props(Full = "Skagerrak", Jname = "スカゲラク海峡", Kind = "Water"))]
    Ska = 60,
    #[strum(props(Full = "Smyrna", Jname = "スミルナ", Kind = "Coast", Supply = "true", Home = "Turkey"))]
    Smy = 61,
    #[strum(props(Full = "Spain", Jname = "スペイン", Kind = "Coast", Supply = "true"))]
    Spa = 62,
    #[strum(props(Full = "Spain(NC)", Jname = "スペイン(NC)", Kind = "Coast"))]
    SpaNc = 63,
    #[strum(props(Full = "Spain(SC)", Jname = "スペイン(SC)", Kind = "Coast"))]
    SpaSc = 64,
    #[strum(props(Full = "St. Petersburg", Jname = "サンクトペテルブルク", Kind = "Coast", Supply = "true", Home = "Russia"))]
    Stp = 65,
    #[strum(props(Full = "St. Petersburg(NC)", Jname = "サンクトペテルブルク(NC)", Kind = "Coast"))]
    StpNc = 66,
    #[strum(props(Full = "St. Petersburg(SC)", Jname = "サンクトペテルブルク(SC)", Kind = "Coast"))]
    StpSc = 67,
    #[strum(props(Full = "Sweden", Jname = "スウェーデン", Kind = "Coast", Supply = "true"))]
    Swe = 68,
    #[strum(props(Full = "Syria", Jname = "シリア", Kind = "Coast", Home = "Turkey"))]
    Syr = 69,
    #[strum(props(Full = "Trieste", Jname = "トリエステ", Kind = "Coast", Supply = "true", Home = "Austria"))]
    Tri = 70,
    #[strum(props(Full = "Tunis", Jname = "チュニス", Kind = "Coast", Supply = "true"))]
    Tun = 71,
    #[strum(props(Full = "Tuscany", Jname = "トスカーナ", Kind = "Coast", Home = "Italy"))]
    Tus = 72,
    #[strum(props(Full = "Tyrolia", Jname = "ティロル", Kind = "Inland", Home = "Austria"))]
    Tyr = 73,
    #[strum(props(Full = "Tyrrhenian Water", Jname = "ティレニア海", Kind = "Water"))]
    Tys = 74,
    #[strum(props(Full = "Ukraine", Jname = "ウクライナ", Kind = "Inland", Home = "Russia"))]
    Ukr = 75,
    #[strum(props(Full = "Venice", Jname = "ヴェネツィア", Kind = "Coast", Supply = "true", Home = "Italy"))]
    Ven = 76,
    #[strum(props(Full = "Vienna", Jname = "ウィーン", Kind = "Inland", Supply = "true", Home = "Austria"))]
    Vie = 77,
    #[strum(props(Full = "Wales", Jname = "ウェールズ", Kind = "Coast", Home = "England"))]
    Wal = 78,
    #[strum(props(Full = "Warsaw", Jname = "ワルシャワ", Kind = "Inland", Supply = "true", Home = "Russia"))]
    War = 79,
    #[strum(props(Full = "Western Mediterranean", Jname = "西地中海", Kind = "Water"))]
    Wes = 80,
    #[strum(props(Full = "Yorkshire", Jname = "ヨークシャー", Kind = "Coast", Home = "England"))]
    Yor = 81,
}

/// 地域のロジック
impl Province {
    /// 地域コードから地域を取得する
    pub fn from_code(code: &str) -> Option<Self> {
        Self::from_str(code).ok()
    }

    /// 全ての地域を返す
    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    /// 地域名を返す
    pub fn name(&self) -> &str {
        self.get_str("Full").unwrap_or("Unknown")
    }

    /// 地域の日本語名を返す
    pub fn jname(&self) -> &str {
        self.get_str("Jname").unwrap_or("不明")
    }

    /// 地域のシンボルを返す
    pub fn symbol(&self) -> &str {
        self.as_ref()
    }

    /// 地域が補給可能かどうかを返す
    pub fn is_suppliable(&self) -> bool {
        self.get_str("Supply") == Some("true")
    }

    /// 地域の地形を返す
    pub fn kind(&self) -> &str {
        self.get_str("Kind").unwrap_or("Unknown")
    }

    /// 地域が水域かどうかを返す
    pub fn is_water(&self) -> bool {
        self.kind() == "Water"
    }

    /// 地域が海岸かどうかを返す
    pub fn is_coast(&self) -> bool {
        self.kind() == "Coast"
    }

    /// 地域が内陸かどうかを返す
    pub fn is_inland(&self) -> bool {
        self.kind() == "Inland"
    }

    /// 地域のホーム国を返す
    pub fn home_power(&self) -> Option<Power> {
        self.get_str("Home").and_then(|s| s.parse::<Power>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_province_props() {
        let p = Province::Par;
        assert_eq!(p.jname(), "パリ");
        assert_eq!(p.name(), "Paris");
        assert!(p.is_inland());
        assert!(p.is_suppliable());
        assert_eq!(p.home_power(), Some(Power::France));

        let water = Province::Adr;
        assert!(water.is_water());
        assert!(!water.is_suppliable());
    }

    #[test]
    fn test_province_from_code() {
        let p = Province::from_code("par").unwrap();
        assert_eq!(p.jname(), "パリ");
        assert_eq!(p.name(), "Paris");
        assert!(p.is_inland());
        assert!(p.is_suppliable());
        assert_eq!(p.home_power(), Some(Power::France));
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
