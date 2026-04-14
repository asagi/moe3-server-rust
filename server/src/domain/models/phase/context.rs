// models
use super::Phase;

// external crates
use serde::Deserialize;
use serde::Serialize;

/// フェイズのコンテキストと終了結果
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct PhaseContext {
    phases: Vec<Phase>,
}

impl PhaseContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self { phases: Vec::new() }
    }

    pub(crate) fn finalize(&mut self, latest_phase: &Phase) -> PhaseCloseResult {
        self.phases.push(latest_phase.clone());
        self.to_result()
    }

    pub(crate) fn push_phase(&mut self, phase: Phase) {
        self.phases.push(phase);
    }

    #[allow(dead_code)]
    pub(crate) fn pop_phase(&mut self) -> Option<Phase> {
        self.phases.pop()
    }

    fn to_result(&self) -> PhaseCloseResult {
        PhaseCloseResult {}
    }
}

/// フェイズの終了結果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCloseResult {}
