// external crates
use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Regulation {
    pub(crate) face_type: FaceType,
    pub(crate) progress_mode: ProgressMode,
    pub(crate) duration_type: DurationType,
    pub(crate) start_date: NaiveDate,
    pub(crate) first_period_hour: u8,
}

/// 画像タイプ
/// - Girls: 娘
/// - Flags: 旗
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FaceType {
    Girls = 1,
    Flags = 2,
}

/// 進行モード
/// - Scheduled: 定時進行
/// - Consensus: 参加者の合意で即時進行
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ProgressMode {
    Scheduled = 1,
    Consensus = 2,
}

/// 期間
/// - Short: 短期
/// - Normal: 通常期間
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DurationType {
    Short = 1,
    Normal = 2,
}

impl Regulation {
    #[allow(dead_code)]
    pub(crate) fn new(
        face_type: FaceType,
        progress_mode: ProgressMode,
        duration_type: DurationType,
        start_date: NaiveDate,
        first_period_hour: u8,
    ) -> Self {
        Self {
            face_type,
            progress_mode,
            duration_type,
            start_date,
            first_period_hour: if (0..24).contains(&first_period_hour) {
                first_period_hour
            } else {
                0
            },
        }
    }
}

impl From<i32> for FaceType {
    fn from(value: i32) -> Self {
        match value {
            1 => FaceType::Girls,
            2 => FaceType::Flags,
            _ => FaceType::Girls,
        }
    }
}

impl From<i32> for ProgressMode {
    fn from(value: i32) -> Self {
        match value {
            1 => ProgressMode::Scheduled,
            2 => ProgressMode::Consensus,
            _ => ProgressMode::Scheduled,
        }
    }
}

impl From<i32> for DurationType {
    fn from(value: i32) -> Self {
        match value {
            1 => DurationType::Short,
            2 => DurationType::Normal,
            _ => DurationType::Normal,
        }
    }
}
