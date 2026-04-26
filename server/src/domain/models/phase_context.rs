// ============================================================================
// imports
// ============================================================================

use std::collections::VecDeque;

use strum::IntoEnumIterator;

use super::Phase;
use super::Power;

// ============================================================================
// definitions
// ============================================================================

///
/// フェイズ更新コンテキストの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PhaseContext {
    active_powers: Vec<Power>,
    phases: VecDeque<Phase>,
    is_draw: bool,
    is_solo: bool,
}

/// フェイズ更新コンテキストの構造体の実装
impl PhaseContext {
    pub(crate) fn new() -> Self {
        Self {
            active_powers: Power::iter().collect(),
            phases: VecDeque::new(),
            is_draw: false,
            is_solo: false,
        }
    }

    ///
    /// 全滅した国や無政府の国を指定し除外する
    ///
    pub(crate) fn remove_power(&mut self, power: &Power) {
        self.active_powers.retain(|p| p != power);
    }

    ///
    /// 活動状態にある国を返却する
    ///
    pub(crate) fn active_powers(&self) -> &[Power] {
        &self.active_powers
    }

    ///
    /// フェイズを追加する
    ///
    pub(crate) fn push_phase(&mut self, phase: Phase) {
        self.phases.push_back(phase);
    }

    ///
    /// 最後のフェイズを削除する
    ///
    #[allow(dead_code)]
    pub(crate) fn pop_phase(&mut self) -> Option<Phase> {
        self.phases.pop_back()
    }

    ///
    /// 全フェイズを返却する
    ///
    pub(crate) fn phases(&self) -> &VecDeque<Phase> {
        &self.phases
    }

    ///
    /// 和平終了フラグの状態を返却する
    ///
    pub(crate) fn is_draw(&self) -> bool {
        self.is_draw
    }

    ///
    /// 制覇終了フラグの状態を返却する
    ///
    pub(crate) fn is_solo(&self) -> bool {
        self.is_solo
    }

    ///
    /// 和平終了フラグを立てる
    ///
    pub(crate) fn set_draw(&mut self) {
        self.is_draw = true;
    }

    ///
    /// 制覇終了フラグを立てる
    ///
    pub(crate) fn set_solo(&mut self) {
        self.is_solo = true;
    }

    ///
    /// ゲーム終了フラグの状態を返却する
    ///
    pub(crate) fn is_finished(&self) -> bool {
        self.is_draw || self.is_solo
    }
}
