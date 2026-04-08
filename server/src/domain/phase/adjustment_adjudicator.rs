use crate::domain::order::Order;
use crate::domain::phase::PhaseContext;

pub struct AdjustmentAdjudicator;

impl AdjustmentAdjudicator {
    pub(crate) fn validate_build_orders(_orders: &mut [Order], _context: &PhaseContext) {
        // TODO
    }

    pub(crate) fn validate_disband_orders(_orders: &mut [Order]) {
        // TODO
    }
}
