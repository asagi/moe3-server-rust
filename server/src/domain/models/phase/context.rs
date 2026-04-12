use super::Phase;
use super::Unit;
use serde::Deserialize;
use serde::Serialize;

/// フェイズのコンテキストと終了結果
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PhaseContext {
    pub phases: Vec<Phase>,
    pub(crate) standoff_codes: Vec<&'static str>,
    pub(crate) last_resolved_units: Vec<Unit>,
}

impl PhaseContext {
    pub fn new() -> Self {
        Self {
            phases: Vec::new(),
            standoff_codes: Vec::new(),
            last_resolved_units: Vec::new(),
        }
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
