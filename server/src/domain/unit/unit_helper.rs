use super::super::power::Power;
use super::super::province::Province;
use super::super::territory::Territory;
use super::super::unit::Unit;

pub trait UnitHelper {
    fn collect_units_for_civil_disorder(&self, power: &Power, territories: &[Territory]) -> Vec<Unit>;
}

impl UnitHelper for [Unit] {
    /// 指定した国のユニットについて下記の条件に基づいて取得する
    /// - 直近の自国が保有する補給都市から距離が遠い順
    /// - 距離が同じ場合は陸軍優先
    /// - 距離と兵種が同じ場合は地域名昇順優先
    fn collect_units_for_civil_disorder(&self, power: &Power, territories: &[Territory]) -> Vec<Unit> {
        let mut units: Vec<Unit> = self.iter().filter(|u| &u.power == power).copied().collect();
        units.sort_by(|&a, &b| {
            let dist_a = territories
                .iter()
                .filter(|t| t.power() == power)
                .map(|t| Province::distance(&a.province.code()[..3], t.code()))
                .min()
                .unwrap_or(0);

            let dist_b = territories
                .iter()
                .filter(|t| t.power() == power)
                .map(|t| Province::distance(&b.province.code()[..3], t.code()))
                .min()
                .unwrap_or(0);

            dist_b
                .cmp(&dist_a)
                .then_with(|| b.is_fleet().cmp(&a.is_fleet()))
                .then_with(|| {
                    a.province
                        .full_name()
                        .to_lowercase()
                        .cmp(&b.province.full_name().to_lowercase())
                })
        });
        units
    }
}
