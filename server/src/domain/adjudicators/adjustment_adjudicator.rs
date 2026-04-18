// models
use super::Path;
use super::Phase;
use super::Power;
use super::Province;

// helpers
use super::AdjustmentOrderHelper;
use super::UnitHelper;

// external crates
use strum::IntoEnumIterator;

pub struct AdjustmentAdjudicator;

impl AdjustmentAdjudicator {
    /// 増設命令の検証
    pub(crate) fn validate_build_orders(current_phase: &mut Phase) {
        for p in Power::iter() {
            // 増設余力算出
            let sc_count = current_phase.count_supply_centers(&p);
            let unit_count = current_phase.count_units(&p);
            let adjustment_capacity = sc_count as isize - unit_count as isize;

            // 増設余力がない場合はスキップ
            if adjustment_capacity < 1 {
                continue;
            }

            let mut remaining = adjustment_capacity as usize;
            while remaining > 0 {
                let Some(i) = current_phase.orders().get_unresolved_build_idxs_by_power(&p) else {
                    break;
                };
                let build_order = current_phase.orders()[i];

                // 増設指定地域が本国補給都市であること
                if !Province::is_home_sc(build_order.location().code(), &p) {
                    current_phase.orders_mut()[i].set_invalid();
                    continue;
                }

                // 増設指定地域が所有されていること
                if !current_phase
                    .territories()
                    .iter()
                    .any(|t| t.code() == build_order.location().code() && t.power() == &p)
                {
                    current_phase.orders_mut()[i].set_invalid();
                    continue;
                }

                // 増設指定地域にユニットが存在していないこと
                if current_phase
                    .units()
                    .iter()
                    .any(|u| u.location().code() == build_order.location().code())
                {
                    current_phase.orders_mut()[i].set_invalid();
                    continue;
                }

                // 増設指定地域がユニットの種類に適合していること
                if !Path::can_unit_exist_at(&build_order.unit, build_order.location().code_with_coast()) {
                    current_phase.orders_mut()[i].set_invalid();
                    continue;
                }

                current_phase.orders_mut()[i].set_valid();
                remaining -= 1;
                continue;
            }
        }
    }

    /// 解体命令の検証
    pub(crate) fn validate_disband_orders(current_phase: &mut Phase) {
        let mut copied_units = current_phase.units().to_vec();

        for p in Power::iter() {
            let sc_count = current_phase.count_supply_centers(&p);
            let unit_count = current_phase.count_units(&p);
            let adjustment_capacity = unit_count as isize - sc_count as isize;

            // 解体の必要がない場合はスキップ
            if adjustment_capacity < 1 {
                continue;
            }

            let mut remaining = adjustment_capacity as usize;
            while remaining > 0 {
                let Some(i) = current_phase.orders().get_unresolved_disband_idxs_by_power(&p) else {
                    break;
                };
                let disband_order = current_phase.orders()[i];

                // 解体指定対象の自国ユニットが存在すること
                if copied_units.iter().any(|u| u == &disband_order.unit) {
                    copied_units.retain(|&u| u != disband_order.unit);
                    current_phase.orders_mut()[i].set_valid();
                    remaining -= 1;
                    continue;
                }

                current_phase.orders_mut()[i].set_invalid();
            }

            // 解体命令が足りない場合の処理
            let disband_candidates = &mut copied_units.collect_units_for_civil_disorder(&p, current_phase.territories());
            while remaining > 0 {
                let unit = disband_candidates.remove(0);
                current_phase.orders_mut().push(unit.disband().set_valid());
                remaining -= 1;
                continue;
            }
        }
    }

    /// 未処理命令をすべて無効判定
    pub(crate) fn invalidate_unresolved_orders(current_phase: &mut Phase) {
        for idx in current_phase.orders().collect_unresolved_adjustment_idxs() {
            current_phase.orders_mut()[idx].set_invalid();
        }
    }
}
