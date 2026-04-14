use crate::domain::models::power::*;
use crate::domain::models::province::*;
use crate::domain::models::unit::*;

/// 地域オブジェクト生成
pub(crate) fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 陸軍オブジェクト生成
pub(crate) fn a(symbol: &str, location_code: &str) -> Unit {
    Unit::new_army(Power::from_symbol(symbol).expect("valid power symobl"), p(location_code))
}

/// 海軍オブジェクト生成
pub(crate) fn f(symbol: &str, location_code: &str) -> Unit {
    Unit::new_fleet(Power::from_symbol(symbol).expect("valid power symobl"), p(location_code))
}
