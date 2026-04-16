// models
use super::Phase;

// standard library
use std::collections::VecDeque;

/// フェイズ遷移のコンテキスト
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct PhaseContext {
    phases: VecDeque<Phase>,
    is_draw: bool,
}

impl PhaseContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            phases: VecDeque::new(),
            is_draw: false,
        }
    }

    pub(crate) fn push_phase(&mut self, phase: Phase) {
        self.phases.push_front(phase);
    }

    #[allow(dead_code)]
    pub(crate) fn pop_phase(&mut self) -> Option<Phase> {
        self.phases.pop_front()
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
