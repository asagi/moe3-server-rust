// ============================================================================
// imports
// ============================================================================

use super::Power;
use super::Province;
use super::Territory;
use super::Unit;

// ============================================================================
// definitions
// ============================================================================

/// 地域オブジェクト生成
pub(crate) fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 陸軍オブジェクト生成
pub(crate) fn a(symbol: &str, location_code: &str) -> Unit {
    let power = Power::from_symbol(symbol).unwrap_or_else(|| panic!("invalid power symbol: {}", symbol));
    Unit::new_army(power, p(location_code))
}

/// 海軍オブジェクト生成
pub(crate) fn f(symbol: &str, location_code: &str) -> Unit {
    let power = Power::from_symbol(symbol).unwrap_or_else(|| panic!("invalid power symbol: {}", symbol));
    Unit::new_fleet(power, p(location_code))
}

/// 占領情報オブジェクト生成
pub(crate) fn t(symbol: &str, location_code: &str) -> Territory {
    let power = Power::from_symbol(symbol).unwrap_or_else(|| panic!("invalid power symbol: {}", symbol));
    Territory::new(power, location_code)
}
