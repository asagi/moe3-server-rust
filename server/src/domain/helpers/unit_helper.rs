// models
use super::Power;
use super::Province;
use super::Territory;
use super::Unit;

pub trait UnitHelper {
    fn collect_all_idxs(&self) -> Vec<usize>;
    fn collect_units_for_civil_disorder(&self, power: &Power, territories: &[Territory]) -> Vec<Unit>;
}

impl UnitHelper for [Unit] {
    /// すべてのユニットのインデックスを取得する
    fn collect_all_idxs(&self) -> Vec<usize> {
        self.iter().enumerate().map(|(idx, _)| idx).collect::<Vec<usize>>()
    }

    /// 指定した国のユニットについて下記の条件に基づいて取得する
    /// - 直近の自国が保有する補給都市から距離が遠い順
    /// - 距離が同じ場合は海軍優先
    /// - 距離と兵種が同じ場合は地域名昇順優先
    fn collect_units_for_civil_disorder(&self, power: &Power, territories: &[Territory]) -> Vec<Unit> {
        let mut units: Vec<Unit> = self.iter().filter(|u| &u.power() == power).copied().collect();
        units.sort_by(|&a, &b| {
            let dist_a = territories
                .iter()
                .filter(|t| t.power() == power)
                .map(|t| Province::distance(&a.location().code()[..3], t.code()))
                .min()
                .unwrap_or(0);

            let dist_b = territories
                .iter()
                .filter(|t| t.power() == power)
                .map(|t| Province::distance(&b.location().code()[..3], t.code()))
                .min()
                .unwrap_or(0);

            dist_b
                .cmp(&dist_a)
                .then_with(|| b.is_fleet().cmp(&a.is_fleet()))
                .then_with(|| {
                    a.location()
                        .full_name()
                        .to_lowercase()
                        .cmp(&b.location().full_name().to_lowercase())
                })
        });
        units
    }
}
