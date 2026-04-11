use crate::domain::path::Path;
use crate::domain::phase::Phase;
use crate::domain::phase::adjustment_order_helper::AdjustmentOrderHelper;
use crate::domain::power::Power;
use crate::domain::province::Province;
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
                let Some(i) = current_phase.data.orders.get_unresolved_build_idxs_by_power(&p) else {
                    break;
                };
                let build_order = current_phase.data.orders[i];

                // 増設指定地域が本国補給都市であること
                if !Province::is_home_sc(&build_order.location().code()[..3], &p) {
                    current_phase.data.orders[i].set_invalid();
                    continue;
                }

                // 増設指定地域が所有されていること
                if !current_phase
                    .data
                    .territories
                    .iter()
                    .any(|t| t.code()[..3] == build_order.location().code()[..3] && t.power() == &p)
                {
                    current_phase.data.orders[i].set_invalid();
                    continue;
                }

                // 増設指定地域にユニットが存在していないこと
                if current_phase
                    .data
                    .units
                    .iter()
                    .any(|u| u.province.code()[..3] == build_order.location().code()[..3])
                {
                    current_phase.data.orders[i].set_invalid();
                    continue;
                }

                // 増設指定地域がユニットの種類に適合していること
                if !Path::can_unit_exist_at(&build_order.unit, build_order.location().code()) {
                    current_phase.data.orders[i].set_invalid();
                    continue;
                }

                current_phase.data.orders[i].set_valid();
                remaining -= 1;
                continue;
            }
        }
    }

    /// 解体命令の検証
    pub(crate) fn validate_disband_orders(current_phase: &mut Phase) {
        for p in Power::iter() {
            // 解体必要数算出
            let sc_count = current_phase.count_supply_centers(&p);
            let unit_count = current_phase.count_units(&p);
            let adjustment_capacity = unit_count as isize - sc_count as isize;

            // 解体の必要がない場合はスキップ
            if adjustment_capacity < 1 {
                continue;
            }

            let mut remaining = adjustment_capacity as usize;
            while remaining > 0 {
                let Some(i) = current_phase.data.orders.get_unresolved_disband_idxs_by_power(&p) else {
                    break;
                };
                let disband_order = current_phase.data.orders[i];

                // 解体指定対象の自国ユニットが存在すること
                if current_phase
                    .data
                    .units
                    .iter()
                    .any(|u| u.province.code()[..3] == disband_order.location().code()[..3] && u.power == p)
                {
                    current_phase.data.orders[i].set_valid();
                    remaining -= 1;
                    continue;
                }

                current_phase.data.orders[i].set_invalid();
            }
        }
    }

    /// 未処理命令をすべて無効判定
    pub(crate) fn invalidate_unresolved_orders(current_phase: &mut Phase) {
        for idx in current_phase.data.orders.collect_unresolved_order_idxs() {
            current_phase.data.orders[idx].set_invalid();
        }
    }
}
