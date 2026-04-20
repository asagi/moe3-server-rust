#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// external crates
use chrono::NaiveDate;
use std::fmt;

// ============================================================================
// definitions
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Regulation {
    pub(crate) face_type: FaceType,
    pub(crate) progress_mode: ProgressMode,
    pub(crate) duration_type: DurationType,
    pub(crate) start_date: NaiveDate,
    pub(crate) first_period_hour: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegulationError {
    FirstPeriodHourOutOfRange(u8),
    UnknownFaceType(i32),
    UnknownProgressMode(i32),
    UnknownDurationType(i32),
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
}

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

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
}
