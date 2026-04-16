// models
use super::Phase;
use super::Power;

// standard library
use std::collections::VecDeque;

/// フェイズ遷移のコンテキスト
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PhaseContext {
    active_powers: Vec<Power>,
    phases: VecDeque<Phase>,
    is_draw: bool,
}

impl PhaseContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            active_powers: vec![
                Power::Austria,
                Power::England,
                Power::France,
                Power::Germany,
                Power::Italy,
                Power::Russia,
                Power::Turkey,
            ],
            phases: VecDeque::new(),
            is_draw: false,
        }
    }

    /// 全滅した国や無政府の国を指定し除外する
    #[allow(dead_code)]
    pub(crate) fn remove_power(&mut self, power: &Power) {
        self.active_powers.retain(|p| p != power);
    }

    pub(crate) fn active_powers(&self) -> &Vec<Power> {
        &self.active_powers
    }

    pub(crate) fn push_phase(&mut self, phase: Phase) {
        self.phases.push_back(phase);
    }

    #[cfg(test)]
    pub(crate) fn pop_phase(&mut self) -> Option<Phase> {
        self.phases.pop_back()
    }

    #[allow(dead_code)]
    pub(crate) fn shift_phase(&mut self) -> Option<Phase> {
        self.phases.pop_front()
    }

    #[cfg(test)]
    pub(crate) fn phases(&self) -> &VecDeque<Phase> {
        &self.phases
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self, idx: usize) -> Option<&Phase> {
        self.phases.get(idx)
    }

    pub(crate) fn is_draw(&self) -> bool {
        self.is_draw
    }

    #[allow(dead_code)]
    pub(crate) fn set_draw(&mut self) {
        self.is_draw = true;
    }
}
