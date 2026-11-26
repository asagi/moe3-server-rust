// ============================================================================
// imports
// ============================================================================

use std::fmt;

use serde_json::Value;
use serde_json::json;
use uuid::Uuid;

use super::Game;
use super::Power;
use super::Province;
use super::Unit;
use super::User;

// ============================================================================
// definitions
// ============================================================================

///
/// メッセージ種別の列挙体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MessageKind {
    Public(PublicPress),
    Confidential(ConfidentialLetter),
    Personal(PersonalNote),
    Ghost(GhostTalk),
    System(SystemNotice),
}

/// メッセージ種別の列挙体の実装
impl MessageKind {
    /// 公式声明を生成する
    pub(crate) fn public() -> Self {
        Self::Public(PublicPress {})
    }

    /// 宛先を指定して機密書簡を生成する
    pub(crate) fn confidential(recipients: Vec<Power>) -> Self {
        Self::Confidential(ConfidentialLetter { recipients })
    }

    /// 独り言を生成する
    pub(crate) fn personal() -> Self {
        Self::Personal(PersonalNote {})
    }

    /// 亡国会話を生成する
    pub(crate) fn ghost() -> Self {
        Self::Ghost(GhostTalk {})
    }

    /// システム通知を生成する
    pub(crate) fn system(catalog: SystemNoticeCatalog) -> Self {
        Self::System(SystemNotice { catalog })
    }
}

///
/// メッセージの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Message {
    // 送信者が存在しない場合は、システムメッセージであることを意味する
    pub(crate) sender: Option<Power>,
    // 帰属ターン（例： "ready", "1901s", "1901f", ..., "debrief"）
    pub(crate) turn: String,
    // 本文
    pub(crate) context: String,
    // メッセージ種別
    pub(crate) kind: MessageKind,
}

///
/// 公式声明の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PublicPress {}

///
/// 機密書簡の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConfidentialLetter {
    pub(crate) recipients: Vec<Power>,
}

///
/// 独り言の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PersonalNote {}

///
/// 亡国会話の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GhostTalk {}

///
/// システム通知の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SystemNotice {
    pub(crate) catalog: SystemNoticeCatalog,
}

///
/// システムメッセージ定義の列挙体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SystemNoticeCatalog {
    GameCreated {
        user: User,
    },
    PlayerJoined {
        user: User,
    },
    Ready,
    Aborted,
    StartSeason {
        // ターン文字列（例: "1901s"）。本文では日本語のシーズン表記（例: "1901 年春"）にする
        season: String,
    },
    DrawProposed,
    OwnerAbsent,
    DrawRescinded,
    Solo {
        power: Power,
    },
    Draw,
    Closed,
    UnitPlaced {
        unit: Unit,
    },
    UnitReplaced {
        old_unit: Unit,
        new_unit: Unit,
    },
    UnitRemoved {
        unit: Unit,
    },
    TerritorySet {
        province: Province,
        power: Power,
    },
    TerritoryReplaced {
        province: Province,
        old_power: Power,
        new_power: Power,
    },
    TerritoryReleased {
        province: Province,
        old_power: Power,
    },
    ProgressModeChanged,
    ProgressConsented {
        power: Power,
    },
    ProgressConsensusRescinded {
        power: Power,
    },
    ProgressConsensusReached,
    NextUpdateAtChanged {
        next_update_at: String,
    },
}

/// システムメッセージ定義の列挙体の fmt::Display トレイト実装
impl fmt::Display for SystemNoticeCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameCreated { user } => {
                write!(f, "@{} が募集を開始しました。", user.username)
            }
            Self::PlayerJoined { user } => {
                write!(f, "@{} が参加を表明しました。", user.username)
            }
            Self::Ready => {
                write!(f, "プレイヤーが揃い、担当国が割り当てられました。")
            }
            Self::Aborted => {
                write!(f, "プレイヤーが揃わなかったため、募集を終了します。")
            }
            Self::StartSeason { season } => {
                let label = Game::season_label_from_turn(season).unwrap_or_else(|| season.clone());
                write!(f, "{} のメインフェイズが開始されました。", label)
            }
            Self::DrawProposed => {
                write!(f, "卓主によって講和が宣言されました。")
            }
            Self::OwnerAbsent => {
                write!(f, "卓主が消息不明のため、自動的に講和の手続きが進められます。")
            }
            Self::DrawRescinded => {
                write!(f, "卓主によって講和が撤回されました。")
            }
            Self::Solo { power } => {
                write!(f, "{} による制覇が達成されました。お疲れさまでした。", power.name())
            }
            Self::Draw => {
                write!(f, "講和が成立しました。お疲れさまでした。")
            }
            Self::Closed => {
                write!(f, "卓が閉鎖されました。")
            }
            Self::UnitPlaced { unit } => {
                write!(f, "{} {} が配置されました。", unit.power.adjective(), unit.label())
            }
            Self::UnitReplaced { old_unit, new_unit } => {
                write!(
                    f,
                    "{} {} が {} {} に変更されました。",
                    old_unit.power.adjective(),
                    old_unit.label(),
                    new_unit.power.adjective(),
                    new_unit.label()
                )
            }
            Self::UnitRemoved { unit } => {
                write!(f, "{} {} が除去されました。", unit.power.adjective(), unit.label())
            }
            Self::TerritorySet { province, power } => {
                write!(f, "{} の保有国が {} に変更されました。", province.jname(), power.name())
            }
            Self::TerritoryReplaced {
                province,
                old_power,
                new_power,
            } => {
                write!(
                    f,
                    "{} の保有国が {} から {} に変更されました。",
                    province.jname(),
                    old_power.name(),
                    new_power.name()
                )
            }
            Self::TerritoryReleased { province, old_power } => {
                write!(
                    f,
                    "{} が保有していた {} が解放されました。",
                    old_power.name(),
                    province.jname()
                )
            }
            Self::ProgressModeChanged => {
                write!(f, "進行モードが定時進行から合意進行に変更されました。")
            }
            Self::ProgressConsented { power } => {
                write!(f, "{} が即時進行に合意しました。", power.name())
            }
            Self::ProgressConsensusRescinded { power } => {
                write!(f, "{} が即時進行への合意を撤回しました。", power.name())
            }
            Self::ProgressConsensusReached => {
                write!(f, "全ての生存国の合意を確認しました。メインフェイズをただちに終了します。")
            }
            Self::NextUpdateAtChanged { next_update_at } => {
                write!(f, "次回更新時刻が {} に変更されました。", next_update_at)
            }
        }
    }
}

/// システムメッセージ定義の列挙体の実装
impl SystemNoticeCatalog {
    ///
    /// 通知の種類（バリアント名の snake_case）を返す
    ///
    pub(crate) fn notice_type(&self) -> &'static str {
        match self {
            Self::GameCreated { .. } => "game_created",
            Self::PlayerJoined { .. } => "player_joined",
            Self::Ready => "ready",
            Self::Aborted => "aborted",
            Self::StartSeason { .. } => "start_season",
            Self::DrawProposed => "draw_proposed",
            Self::OwnerAbsent => "owner_absent",
            Self::DrawRescinded => "draw_rescinded",
            Self::Solo { .. } => "solo",
            Self::Draw => "draw",
            Self::Closed => "closed",
            Self::UnitPlaced { .. } => "unit_placed",
            Self::UnitReplaced { .. } => "unit_replaced",
            Self::UnitRemoved { .. } => "unit_removed",
            Self::TerritorySet { .. } => "territory_set",
            Self::TerritoryReplaced { .. } => "territory_replaced",
            Self::TerritoryReleased { .. } => "territory_released",
            Self::ProgressModeChanged => "progress_mode_changed",
            Self::ProgressConsented { .. } => "progress_consented",
            Self::ProgressConsensusRescinded { .. } => "progress_consensus_rescinded",
            Self::ProgressConsensusReached => "progress_consensus_reached",
            Self::NextUpdateAtChanged { .. } => "next_update_at_changed",
        }
    }

    ///
    /// 本文の組み立てに使った値を、表示名ではなくコードで返す
    ///
    /// - 国は記号（例: "g"）、地域はコード（例: "mun" / "stp_sc"）、ユニット種別は "a" / "f"
    /// - ユーザーは Discord のユーザー名のみ（数値 ID は含めない）
    ///
    pub(crate) fn notice_params(&self) -> Value {
        match self {
            Self::GameCreated { user } | Self::PlayerJoined { user } => json!({ "username": user.username }),
            Self::StartSeason { season } => json!({ "season": season }),
            Self::Solo { power } | Self::ProgressConsented { power } | Self::ProgressConsensusRescinded { power } => {
                json!({ "power": power.symbol() })
            }
            Self::UnitPlaced { unit } | Self::UnitRemoved { unit } => json!({ "unit": Self::unit_params(unit) }),
            Self::UnitReplaced { old_unit, new_unit } => json!({
                "old_unit": Self::unit_params(old_unit),
                "new_unit": Self::unit_params(new_unit),
            }),
            Self::TerritorySet { province, power } => json!({
                "province": province.code_with_coast(),
                "power": power.symbol(),
            }),
            Self::TerritoryReplaced {
                province,
                old_power,
                new_power,
            } => json!({
                "province": province.code_with_coast(),
                "old_power": old_power.symbol(),
                "new_power": new_power.symbol(),
            }),
            Self::TerritoryReleased { province, old_power } => json!({
                "province": province.code_with_coast(),
                "old_power": old_power.symbol(),
            }),
            Self::NextUpdateAtChanged { next_update_at } => json!({ "next_update_at": next_update_at }),
            Self::Ready
            | Self::Aborted
            | Self::DrawProposed
            | Self::OwnerAbsent
            | Self::DrawRescinded
            | Self::Draw
            | Self::Closed
            | Self::ProgressModeChanged
            | Self::ProgressConsensusReached => json!({}),
        }
    }

    /// ユニットを `GET /board` の `units[]` と同じ形で返す
    fn unit_params(unit: &Unit) -> Value {
        json!({
            "power": unit.power.symbol(),
            "kind": unit.kind_code(),
            "location": unit.location.code_with_coast(),
        })
    }

    ///
    /// 保存済みの本文から、システムメッセージ定義を逆算する
    ///
    /// 種類を保存していなかった頃のメッセージの移行に使う。
    /// 逆算した定義から本文を組み立て直し、元の本文と一致した場合だけ返す。
    /// ユーザーはユーザー名以外を復元できないため、ユーザー名以外はダミー値になる。
    ///
    pub(crate) fn parse_context(context: &str) -> Option<Self> {
        let catalog = Self::parse_context_candidate(context)?;
        (catalog.to_string() == context).then_some(catalog)
    }

    /// 本文から定義の候補を組み立てる（本文との一致確認は呼び出し元で行う）
    fn parse_context_candidate(context: &str) -> Option<Self> {
        // 値を持たない定義は、本文が完全に一致するかで判定する
        let fixed = [
            Self::Ready,
            Self::Aborted,
            Self::DrawProposed,
            Self::OwnerAbsent,
            Self::DrawRescinded,
            Self::Draw,
            Self::Closed,
            Self::ProgressModeChanged,
            Self::ProgressConsensusReached,
        ];
        if let Some(catalog) = fixed.into_iter().find(|catalog| catalog.to_string() == context) {
            return Some(catalog);
        }

        if let Some(username) = Self::between(context, "@", " が募集を開始しました。") {
            return Some(Self::GameCreated {
                user: Self::user_with_name(username),
            });
        }
        if let Some(username) = Self::between(context, "@", " が参加を表明しました。") {
            return Some(Self::PlayerJoined {
                user: Self::user_with_name(username),
            });
        }
        if let Some(label) = Self::between(context, "", " のメインフェイズが開始されました。") {
            return Some(Self::StartSeason {
                season: Self::turn_by_season_label(label)?,
            });
        }
        if let Some(name) = Self::between(context, "", " による制覇が達成されました。お疲れさまでした。") {
            return Some(Self::Solo {
                power: Self::power_by_name(name)?,
            });
        }
        if let Some(name) = Self::between(context, "", " が即時進行に合意しました。") {
            return Some(Self::ProgressConsented {
                power: Self::power_by_name(name)?,
            });
        }
        if let Some(name) = Self::between(context, "", " が即時進行への合意を撤回しました。") {
            return Some(Self::ProgressConsensusRescinded {
                power: Self::power_by_name(name)?,
            });
        }
        if let Some(next_update_at) = Self::between(context, "次回更新時刻が ", " に変更されました。") {
            return Some(Self::NextUpdateAtChanged {
                next_update_at: next_update_at.to_string(),
            });
        }
        if let Some(label) = Self::between(context, "", " が配置されました。") {
            return Some(Self::UnitPlaced {
                unit: Self::unit_by_label(label)?,
            });
        }
        if let Some(label) = Self::between(context, "", " が除去されました。") {
            return Some(Self::UnitRemoved {
                unit: Self::unit_by_label(label)?,
            });
        }
        if let Some(rest) = Self::between(context, "", " が解放されました。") {
            let (name, jname) = rest.split_once(" が保有していた ")?;
            return Some(Self::TerritoryReleased {
                province: Self::province_by_jname(jname)?,
                old_power: Self::power_by_name(name)?,
            });
        }
        if let Some(rest) = Self::between(context, "", " に変更されました。") {
            if let Some((jname, powers)) = rest.split_once(" の保有国が ") {
                let province = Self::province_by_jname(jname)?;
                return match powers.split_once(" から ") {
                    Some((old_name, new_name)) => Some(Self::TerritoryReplaced {
                        province,
                        old_power: Self::power_by_name(old_name)?,
                        new_power: Self::power_by_name(new_name)?,
                    }),
                    None => Some(Self::TerritorySet {
                        province,
                        power: Self::power_by_name(powers)?,
                    }),
                };
            }
            let (old_label, new_label) = rest.split_once(" が ")?;
            return Some(Self::UnitReplaced {
                old_unit: Self::unit_by_label(old_label)?,
                new_unit: Self::unit_by_label(new_label)?,
            });
        }

        None
    }

    /// 本文から接頭辞・接尾辞を取り除いた部分を返す（空の場合は None）
    fn between<'a>(context: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
        let value = context.strip_prefix(prefix)?.strip_suffix(suffix)?;
        (!value.is_empty()).then_some(value)
    }

    /// ユーザー名だけを持つユーザーを生成する
    fn user_with_name(username: &str) -> User {
        User {
            uuid: Uuid::nil(),
            discord_user_id: String::new(),
            username: username.to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
        }
    }

    /// 日本語のシーズン表記（例: "1901 年春"）からターン文字列（例: "1901s"）を返す
    fn turn_by_season_label(label: &str) -> Option<String> {
        let (year, season) = label.split_once(" 年")?;
        let year = year.parse::<i32>().ok()?;
        match season {
            "春" => Some(format!("{}s", year)),
            "秋" => Some(format!("{}f", year)),
            _ => None,
        }
    }

    /// 国名（例: "Germany"）から国を返す
    fn power_by_name(name: &str) -> Option<Power> {
        use strum::IntoEnumIterator;
        Power::iter().find(|power| power.name() == name)
    }

    /// 日本語名から地域を返す
    fn province_by_jname(jname: &str) -> Option<Province> {
        Province::all().find(|province| province.jname() == jname)
    }

    /// ユニットの表記（例: "German A Mun"）からユニットを返す
    fn unit_by_label(label: &str) -> Option<Unit> {
        use strum::IntoEnumIterator;
        let mut tokens = label.split(' ');
        let (adjective, symbol, short_name) = (tokens.next()?, tokens.next()?, tokens.next()?);
        if tokens.next().is_some() {
            return None;
        }
        let power = Power::iter().find(|power| power.adjective() == adjective)?;
        let province = Province::all().find(|province| province.short_name() == short_name)?;
        match symbol {
            "A" => Some(Unit::new_army(power, province)),
            "F" => Some(Unit::new_fleet(power, province)),
            _ => None,
        }
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn province(code: &str) -> Province {
        Province::from_code(code).expect("valid province code")
    }

    fn sample_catalogs() -> Vec<SystemNoticeCatalog> {
        vec![
            SystemNoticeCatalog::GameCreated {
                user: SystemNoticeCatalog::user_with_name("alice"),
            },
            SystemNoticeCatalog::PlayerJoined {
                user: SystemNoticeCatalog::user_with_name("bob が参加"),
            },
            SystemNoticeCatalog::Ready,
            SystemNoticeCatalog::Aborted,
            SystemNoticeCatalog::StartSeason {
                season: "1901s".to_string(),
            },
            SystemNoticeCatalog::DrawProposed,
            SystemNoticeCatalog::OwnerAbsent,
            SystemNoticeCatalog::DrawRescinded,
            SystemNoticeCatalog::Solo { power: Power::France },
            SystemNoticeCatalog::Draw,
            SystemNoticeCatalog::Closed,
            SystemNoticeCatalog::UnitPlaced {
                unit: Unit::new_army(Power::Germany, province("mun")),
            },
            SystemNoticeCatalog::UnitReplaced {
                old_unit: Unit::new_army(Power::Austria, province("bud")),
                new_unit: Unit::new_fleet(Power::Russia, province("stp_sc")),
            },
            SystemNoticeCatalog::UnitRemoved {
                unit: Unit::new_fleet(Power::England, province("nth")),
            },
            SystemNoticeCatalog::TerritorySet {
                province: province("mun"),
                power: Power::Germany,
            },
            SystemNoticeCatalog::TerritoryReplaced {
                province: province("mun"),
                old_power: Power::Austria,
                new_power: Power::Germany,
            },
            SystemNoticeCatalog::TerritoryReleased {
                province: province("mun"),
                old_power: Power::Germany,
            },
            SystemNoticeCatalog::ProgressModeChanged,
            SystemNoticeCatalog::ProgressConsented { power: Power::France },
            SystemNoticeCatalog::ProgressConsensusRescinded { power: Power::Turkey },
            SystemNoticeCatalog::ProgressConsensusReached,
            SystemNoticeCatalog::NextUpdateAtChanged {
                next_update_at: "2026-11-27 21:00".to_string(),
            },
        ]
    }

    #[test]
    fn notice_params_use_codes_instead_of_display_names() {
        let unit_placed = SystemNoticeCatalog::UnitPlaced {
            unit: Unit::new_army(Power::Germany, province("mun")),
        };
        assert_eq!(unit_placed.notice_type(), "unit_placed");
        assert_eq!(
            unit_placed.notice_params(),
            json!({ "unit": { "power": "g", "kind": "a", "location": "mun" } })
        );

        let unit_replaced = SystemNoticeCatalog::UnitReplaced {
            old_unit: Unit::new_army(Power::Austria, province("bud")),
            new_unit: Unit::new_fleet(Power::Russia, province("stp_sc")),
        };
        assert_eq!(
            unit_replaced.notice_params(),
            json!({
                "old_unit": { "power": "a", "kind": "a", "location": "bud" },
                "new_unit": { "power": "r", "kind": "f", "location": "stp_sc" },
            })
        );

        let territory_replaced = SystemNoticeCatalog::TerritoryReplaced {
            province: province("mun"),
            old_power: Power::Austria,
            new_power: Power::Germany,
        };
        assert_eq!(territory_replaced.notice_type(), "territory_replaced");
        assert_eq!(
            territory_replaced.notice_params(),
            json!({ "province": "mun", "old_power": "a", "new_power": "g" })
        );

        let solo = SystemNoticeCatalog::Solo { power: Power::France };
        assert_eq!(solo.notice_type(), "solo");
        assert_eq!(solo.notice_params(), json!({ "power": "f" }));

        let created = SystemNoticeCatalog::GameCreated {
            user: User {
                uuid: Uuid::now_v7(),
                discord_user_id: "1001".to_string(),
                username: "alice".to_string(),
                global_name: Some("Alice".to_string()),
                avatar_hash: None,
                avatar_url: None,
            },
        };
        assert_eq!(created.notice_type(), "game_created");
        assert_eq!(created.notice_params(), json!({ "username": "alice" }));

        let start_season = SystemNoticeCatalog::StartSeason {
            season: "1901f".to_string(),
        };
        assert_eq!(start_season.to_string(), "1901 年秋 のメインフェイズが開始されました。");
        assert_eq!(start_season.notice_params(), json!({ "season": "1901f" }));

        assert_eq!(SystemNoticeCatalog::Ready.notice_params(), json!({}));
    }

    #[test]
    fn parse_context_restores_every_catalog() {
        for catalog in sample_catalogs() {
            let context = catalog.to_string();
            let parsed = SystemNoticeCatalog::parse_context(&context).unwrap_or_else(|| panic!("should parse: {}", context));
            assert_eq!(parsed.notice_type(), catalog.notice_type(), "context: {}", context);
            assert_eq!(parsed.notice_params(), catalog.notice_params(), "context: {}", context);
        }
    }

    #[test]
    fn parse_context_returns_none_for_unknown_context() {
        for context in [
            "",
            "未知のメッセージです。",
            "Unknown A Mun が配置されました。",
            "German A Xyz が配置されました。",
            "未知の地域 の保有国が France に変更されました。",
            "Narnia による制覇が達成されました。お疲れさまでした。",
            "1901 年冬 のメインフェイズが開始されました。",
        ] {
            assert!(SystemNoticeCatalog::parse_context(context).is_none(), "context: {}", context);
        }
    }
}
