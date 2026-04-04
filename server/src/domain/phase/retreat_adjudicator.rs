use crate::domain::order::Order;
use crate::domain::phase::retreat_order_helper::RetreatOrderHelper;

pub struct RetreatAdjudicator;

impl RetreatAdjudicator {
    pub(crate) fn validate_retreat_orders(orders: &mut [Order]) {
        let _orders = orders.collect_retreat_indices();
    }
}
