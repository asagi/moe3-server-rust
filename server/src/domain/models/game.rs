// ============================================================================
// imports
// ============================================================================

use chrono::FixedOffset;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Timelike;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::DurationType;
use super::Phase;
use super::PhaseKind;
use super::Player;
use super::ProgressMode;
use super::Regulation;

// ============================================================================
// definitions
// ============================================================================

///
/// 卓の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Game {
    pub uuid: Uuid,
    pub(crate) game_number: Option<i32>,
    pub(crate) regulation: Regulation,
    pub(crate) players: Vec<Player>,
    pub(crate) phases: Vec<Phase>,
    pub(crate) status: GameStatus,
    pub(crate) is_draw: bool,
    pub(crate) is_solo: bool,
    pub(crate) next_update_at: Option<NaiveDateTime>,
}

impl Game {
    pub(crate) fn calculate_next_update(&self, previous_next_update: NaiveDateTime, next_phase: &Phase) -> NaiveDateTime {
        if self.regulation.progress_mode == ProgressMode::Scheduled
            && self.regulation.duration_type == DurationType::Normal
            && Self::is_main_phase(next_phase)
        {
            return Self::next_day_at_first_period_hour(previous_next_update, self.regulation.first_period_hour);
        }

        let duration_minutes = if Self::is_debrief_phase(next_phase) {
            self.regulation.duration_type.debrief_phase_minutes()
        } else if Self::is_sub_phase(next_phase) {
            self.regulation.duration_type.sub_phase_minutes()
        } else {
            self.regulation.duration_type.main_phase_minutes()
        };

        let raw_next_update = previous_next_update + chrono::Duration::minutes(i64::from(duration_minutes));
        Self::ceil_to_5_minutes(raw_next_update)
    }

    fn next_day_at_first_period_hour(previous_next_update: NaiveDateTime, first_period_hour: u8) -> NaiveDateTime {
        let jst = FixedOffset::east_opt(9 * 60 * 60).expect("JST offset should be valid");
        let previous_jst =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(previous_next_update, chrono::Utc).with_timezone(&jst);

        let next_date = previous_jst
            .date_naive()
            .succ_opt()
            .expect("next day should exist for NaiveDate");

        jst.from_local_datetime(
            &next_date
                .and_hms_opt(u32::from(first_period_hour), 0, 0)
                .expect("first_period_hour is validated in Regulation::new"),
        )
        .single()
        .expect("JST local datetime should map uniquely")
        .with_timezone(&chrono::Utc)
        .naive_utc()
    }

    fn is_sub_phase(phase: &Phase) -> bool {
        matches!(
            phase.kind,
            PhaseKind::SpringRetreat(_) | PhaseKind::FallRetreat(_) | PhaseKind::Adjustment(_)
        )
    }

    fn is_main_phase(phase: &Phase) -> bool {
        matches!(phase.kind, PhaseKind::SpringMain(_) | PhaseKind::FallMain(_))
    }

    fn is_debrief_phase(phase: &Phase) -> bool {
        matches!(phase.kind, PhaseKind::Debrief(_))
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
/// 卓のステータスの列挙体
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GameStatus {
    Preparing,
    Ready,
    InProgress,
    Finished,
    Aborted,
    Closed,
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::FaceType;
    use crate::domain::Power;

    fn valid_date() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date")
    }

    fn sample_game(regulation: Regulation) -> Game {
        Game {
            uuid: Uuid::now_v7(),
            game_number: None,
            regulation,
            players: vec![Player {
                user_uuid: Uuid::now_v7(),
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        }
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
        let game = sample_game(regulation);
        let previous_next_update = chrono::NaiveDate::from_ymd_opt(2026, 4, 22)
            .expect("valid date")
            .and_hms_opt(14, 32, 0)
            .expect("valid datetime");

        let next_update = game.calculate_next_update(previous_next_update, &Phase::new_fall_main(1901, 1));

        assert_eq!(
            next_update,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 23)
                .expect("valid date")
                .and_hms_opt(12, 0, 0)
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
        let game = sample_game(regulation);
        let previous_next_update = chrono::NaiveDate::from_ymd_opt(2026, 4, 22)
            .expect("valid date")
            .and_hms_opt(14, 32, 0)
            .expect("valid datetime");

        let next_update = game.calculate_next_update(previous_next_update, &Phase::new_fall_retreat(1901, 1));

        assert_eq!(
            next_update,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 22)
                .expect("valid date")
                .and_hms_opt(14, 50, 0)
                .expect("valid datetime")
        );
    }

    #[test]
    fn calculate_next_update_uses_24_hours_for_debrief_even_on_short_duration() {
        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            valid_date(),
            21,
        )
        .expect("valid regulation");
        let game = sample_game(regulation);
        let previous_next_update = chrono::NaiveDate::from_ymd_opt(2026, 4, 22)
            .expect("valid date")
            .and_hms_opt(14, 32, 0)
            .expect("valid datetime");

        let next_update = game.calculate_next_update(previous_next_update, &Phase::new_debrief(1901, 1));

        assert_eq!(
            next_update,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 23)
                .expect("valid date")
                .and_hms_opt(14, 35, 0)
                .expect("valid datetime")
        );
    }
}
