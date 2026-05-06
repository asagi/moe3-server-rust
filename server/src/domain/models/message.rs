// ============================================================================
// imports
// ============================================================================

use std::fmt;

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
    #[allow(dead_code)]
    Public(PublicPress),
    #[allow(dead_code)]
    Confidential(ConfidentialLetter),
    #[allow(dead_code)]
    Personal(PersonalNote),
    #[allow(dead_code)]
    Ghost(GhostTalk),
    System(SystemNotice),
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
pub(crate) struct SystemNotice {}

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
}

/// システムメッセージ定義の列挙体の fmt::Display トレイト実装
impl fmt::Display for SystemNoticeCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameCreated { user } => {
                write!(f, "{} ({}) が募集を開始しました。", user.username, user.discord_user_id)
            }
            Self::PlayerJoined { user } => {
                write!(f, "{} ({}) が参加を表明しました。", user.username, user.discord_user_id)
            }
            Self::Ready => {
                write!(f, "プレイヤーが揃い、担当国が割り当てられました。")
            }
            Self::Aborted => {
                write!(f, "プレイヤーが揃わなかったため、募集を終了します。")
            }
            Self::StartSeason { season } => {
                write!(f, "{} のメインフェイズが開始されました。", season)
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
        }
    }
}
