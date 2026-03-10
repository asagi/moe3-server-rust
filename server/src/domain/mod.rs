pub mod order;
pub mod power;
pub mod province;
pub mod unit;

pub use power::Power;
pub use province::Province;
pub use unit::Unit;

pub type OrderId = i64;
pub type UnitId = i64;
