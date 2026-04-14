// models
use super::Phase;

// external crates
use serde::Deserialize;
use serde::Serialize;

/// フェイズのコンテキストと終了結果
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PhaseContext {
    pub phases: Vec<Phase>,
}

impl PhaseContext {
    pub fn new() -> Self {
        Self { phases: Vec::new() }
    }

    pub fn finalize(&mut self, latest_phase: &Phase) -> PhaseCloseResult {
        self.phases.push(latest_phase.clone());
        self.to_result()
    }

    fn to_result(&self) -> PhaseCloseResult {
        PhaseCloseResult {}
    }
}

/// フェイズの終了結果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCloseResult {}
