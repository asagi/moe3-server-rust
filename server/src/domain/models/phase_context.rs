#![allow(dead_code)] // TODO: 後で削除する
// ============================================================================
// imports
// ============================================================================

// standard library
use std::collections::VecDeque;

// external crates
use strum::IntoEnumIterator;

// structs
use super::Phase;
use super::Power;

// ============================================================================
// definitions
// ============================================================================

/// フェイズ遷移のコンテキスト
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PhaseContext {
    active_powers: Vec<Power>,
    phases: VecDeque<Phase>,
    is_draw: bool,
    is_solo: bool,
}

impl PhaseContext {
    pub(crate) fn new() -> Self {
        Self {
            active_powers: Power::iter().collect(),
            phases: VecDeque::new(),
            is_draw: false,
            is_solo: false,
        }
    }

    /// 全滅した国や無政府の国を指定し除外する
    pub(crate) fn remove_power(&mut self, power: &Power) {
        self.active_powers.retain(|p| p != power);
    }

    pub(crate) fn active_powers(&self) -> &[Power] {
        &self.active_powers
    }

    pub(crate) fn push_phase(&mut self, phase: Phase) {
        self.phases.push_back(phase);
    }

    pub(crate) fn pop_phase(&mut self) -> Option<Phase> {
        self.phases.pop_back()
    }

    pub(crate) fn phases(&self) -> &VecDeque<Phase> {
        &self.phases
    }

    pub(crate) fn is_draw(&self) -> bool {
        self.is_draw
    }

    pub(crate) fn is_solo(&self) -> bool {
        self.is_solo
    }

    pub(crate) fn set_draw(&mut self) {
        self.is_draw = true;
    }

    pub(crate) fn set_solo(&mut self) {
        self.is_solo = true;
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.is_draw || self.is_solo
    }
}
