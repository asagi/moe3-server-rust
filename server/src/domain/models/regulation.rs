// ============================================================================
// imports
// ============================================================================

use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::Timelike;
use std::fmt;

// ============================================================================
// definitions
// ============================================================================

///
/// 卓レギュレーションの構造体
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Regulation {
    pub(crate) face_type: FaceType,
    pub(crate) progress_mode: ProgressMode,
    pub(crate) duration_type: DurationType,
    pub(crate) start_date: NaiveDate,
    pub(crate) first_period_hour: u8,
}

/// 卓レギュレーションの構造体の実装
impl Regulation {
    pub(crate) fn new(
        face_type: FaceType,
        progress_mode: ProgressMode,
        duration_type: DurationType,
        start_date: NaiveDate,
        first_period_hour: u8,
    ) -> Result<Self, RegulationError> {
        if !(0..24).contains(&first_period_hour) {
            return Err(RegulationError::FirstPeriodHourOutOfRange(first_period_hour));
        }

        Ok(Self {
            face_type,
            progress_mode,
            duration_type,
            start_date,
            first_period_hour,
        })
    }

    ///
    /// 次の更新日時を計算する
    ///
    pub(crate) fn calculate_next_update(self, previous_next_update: NaiveDateTime, next_phase: &super::Phase) -> NaiveDateTime {
        if self.progress_mode == ProgressMode::Scheduled
            && self.duration_type == DurationType::Normal
            && Self::is_main_phase(next_phase)
        {
            return Self::next_day_at_first_period_hour(previous_next_update, self.first_period_hour);
        }

        let duration_minutes = if Self::is_sub_phase(next_phase) {
            self.duration_type.sub_phase_minutes()
        } else {
            self.duration_type.main_phase_minutes()
        };

        let raw_next_update = previous_next_update + chrono::Duration::minutes(i64::from(duration_minutes));
        Self::ceil_to_5_minutes(raw_next_update)
    }

    fn next_day_at_first_period_hour(previous_next_update: NaiveDateTime, first_period_hour: u8) -> NaiveDateTime {
        let next_date = previous_next_update
            .date()
            .succ_opt()
            .expect("next day should exist for NaiveDate");
        next_date
            .and_hms_opt(u32::from(first_period_hour), 0, 0)
            .expect("first_period_hour is validated in Regulation::new")
    }

    fn is_sub_phase(phase: &super::Phase) -> bool {
        matches!(
            phase.kind,
            super::PhaseKind::SpringRetreat(_) | super::PhaseKind::FallRetreat(_) | super::PhaseKind::Adjustment(_)
        )
    }

    fn is_main_phase(phase: &super::Phase) -> bool {
        matches!(phase.kind, super::PhaseKind::SpringMain(_) | super::PhaseKind::FallMain(_))
    }

    fn ceil_to_5_minutes(dt: NaiveDateTime) -> NaiveDateTime {
        let truncated =
            dt - chrono::Duration::seconds(i64::from(dt.second())) - chrono::Duration::nanoseconds(i64::from(dt.nanosecond()));

        let minute_aligned = if dt.second() == 0 && dt.nanosecond() == 0 {
            truncated
        } else {
            truncated + chrono::Duration::minutes(1)
        };

        let remainder = minute_aligned.minute() % 5;
        if remainder == 0 {
            minute_aligned
        } else {
            minute_aligned + chrono::Duration::minutes(i64::from(5 - remainder))
        }
    }
}

///
/// 卓レギュレーションの画像タイプの列挙体
///
/// - Girls: 娘
/// - Flags: 旗
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FaceType {
    Girls = 1,
    Flags = 2,
}

/// 卓レギュレーションの画像タイプの列挙体の実装（TryFrom トレイト）
impl TryFrom<i32> for FaceType {
    type Error = RegulationError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(FaceType::Girls),
            2 => Ok(FaceType::Flags),
            _ => Err(RegulationError::UnknownFaceType(value)),
        }
    }
}

///
/// 卓レギュレーションの進行モードの列挙体
///
/// - Scheduled: 定時進行
/// - Consensus: 参加者の合意で即時進行
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ProgressMode {
    Scheduled = 1,
    Consensus = 2,
}

///
/// 卓レギュレーションの期間タイプの列挙体
///
/// - Short: 短期
/// - Normal: 通常期間
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DurationType {
    Short = 1,
    Normal = 2,
}

impl TryFrom<i32> for ProgressMode {
    type Error = RegulationError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ProgressMode::Scheduled),
            2 => Ok(ProgressMode::Consensus),
            _ => Err(RegulationError::UnknownProgressMode(value)),
        }
    }
}

/// 卓レギュレーションの期間タイプの列挙体の実装
impl DurationType {
    /// メインフェイズの時間（分）
    pub(crate) const fn main_phase_minutes(self) -> u32 {
        match self {
            DurationType::Short => 30,
            DurationType::Normal => 60 * 24,
        }
    }

    /// 撤退調整フェイズの時間（分）
    pub(crate) const fn sub_phase_minutes(self) -> u32 {
        match self {
            DurationType::Short => 10,
            DurationType::Normal => 15,
        }
    }
}

/// 卓レギュレーションの期間タイプの列挙体の実装（TryFrom トレイト）
impl TryFrom<i32> for DurationType {
    type Error = RegulationError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DurationType::Short),
            2 => Ok(DurationType::Normal),
            _ => Err(RegulationError::UnknownDurationType(value)),
        }
    }
}

///
/// 卓レギュレーションのエラーの列挙体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegulationError {
    FirstPeriodHourOutOfRange(u8),
    UnknownFaceType(i32),
    UnknownProgressMode(i32),
    UnknownDurationType(i32),
}

/// 卓レギュレーションのエラーの実装
impl fmt::Display for RegulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegulationError::FirstPeriodHourOutOfRange(v) => write!(f, "invalid first_period_hour: {}", v),
            RegulationError::UnknownFaceType(v) => write!(f, "invalid FaceType value: {}", v),
            RegulationError::UnknownProgressMode(v) => write!(f, "invalid ProgressMode value: {}", v),
            RegulationError::UnknownDurationType(v) => write!(f, "invalid DurationType value: {}", v),
        }
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Phase;

    fn valid_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date")
    }

    #[test]
    fn face_type_try_from_accepts_known_values() {
        assert_eq!(FaceType::try_from(1), Ok(FaceType::Girls));
        assert_eq!(FaceType::try_from(2), Ok(FaceType::Flags));
    }

    #[test]
    fn face_type_try_from_rejects_unknown_values() {
        assert_eq!(FaceType::try_from(0), Err(RegulationError::UnknownFaceType(0)));
        assert_eq!(FaceType::try_from(-1), Err(RegulationError::UnknownFaceType(-1)));
    }

    #[test]
    fn progress_mode_try_from_accepts_known_values() {
        assert_eq!(ProgressMode::try_from(1), Ok(ProgressMode::Scheduled));
        assert_eq!(ProgressMode::try_from(2), Ok(ProgressMode::Consensus));
    }

    #[test]
    fn progress_mode_try_from_rejects_unknown_values() {
        assert_eq!(ProgressMode::try_from(0), Err(RegulationError::UnknownProgressMode(0)));
        assert_eq!(ProgressMode::try_from(-1), Err(RegulationError::UnknownProgressMode(-1)));
    }

    #[test]
    fn duration_type_try_from_accepts_known_values() {
        assert_eq!(DurationType::try_from(1), Ok(DurationType::Short));
        assert_eq!(DurationType::try_from(2), Ok(DurationType::Normal));
    }

    #[test]
    fn duration_type_try_from_rejects_unknown_values() {
        assert_eq!(DurationType::try_from(0), Err(RegulationError::UnknownDurationType(0)));
        assert_eq!(DurationType::try_from(-1), Err(RegulationError::UnknownDurationType(-1)));
    }

    #[test]
    fn duration_type_phase_minutes() {
        assert_eq!(DurationType::Short.main_phase_minutes(), 30);
        assert_eq!(DurationType::Short.sub_phase_minutes(), 10);
        assert_eq!(DurationType::Normal.main_phase_minutes(), 60 * 24);
        assert_eq!(DurationType::Normal.sub_phase_minutes(), 60);
    }

    #[test]
    fn regulation_new_accepts_hour_0_and_23() {
        let r0 = Regulation::new(FaceType::Girls, ProgressMode::Scheduled, DurationType::Short, valid_date(), 0)
            .expect("hour 0 should be valid");
        assert_eq!(r0.first_period_hour, 0);

        let r23 = Regulation::new(
            FaceType::Flags,
            ProgressMode::Consensus,
            DurationType::Normal,
            valid_date(),
            23,
        )
        .expect("hour 23 should be valid");
        assert_eq!(r23.first_period_hour, 23);
    }

    #[test]
    fn regulation_new_rejects_hour_24() {
        assert_eq!(
            Regulation::new(
                FaceType::Girls,
                ProgressMode::Scheduled,
                DurationType::Short,
                valid_date(),
                24,
            ),
            Err(RegulationError::FirstPeriodHourOutOfRange(24)),
        );
    }

    #[test]
    fn calculate_next_update_sets_next_day_first_period_hour_for_scheduled_normal_main_phase() {
        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Normal,
            valid_date(),
            21,
        )
        .expect("valid regulation");
        let previous_next_update = NaiveDate::from_ymd_opt(2026, 4, 22)
            .expect("valid date")
            .and_hms_opt(14, 32, 0)
            .expect("valid datetime");

        let next_update = regulation.calculate_next_update(previous_next_update, &Phase::new_fall_main(1901, 1));

        assert_eq!(
            next_update,
            NaiveDate::from_ymd_opt(2026, 4, 23)
                .expect("valid date")
                .and_hms_opt(21, 0, 0)
                .expect("valid datetime")
        );
    }

    #[test]
    fn calculate_next_update_uses_sub_phase_duration_for_scheduled_normal_sub_phase() {
        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Normal,
            valid_date(),
            21,
        )
        .expect("valid regulation");
        let previous_next_update = NaiveDate::from_ymd_opt(2026, 4, 22)
            .expect("valid date")
            .and_hms_opt(14, 32, 0)
            .expect("valid datetime");

        let next_update = regulation.calculate_next_update(previous_next_update, &Phase::new_fall_retreat(1901, 1));

        assert_eq!(
            next_update,
            NaiveDate::from_ymd_opt(2026, 4, 22)
                .expect("valid date")
                .and_hms_opt(15, 35, 0)
                .expect("valid datetime")
        );
    }
}
