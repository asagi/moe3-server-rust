// ============================================================================
// imports
// ============================================================================

use serde::Deserialize;

// ============================================================================
// definitions
// ============================================================================

///
/// 発言投稿リクエストパラメータボディ構造体
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PostGameMessageRequestBody {
    /// 種別（"public" / "confidential" / "personal"）
    pub kind: String,
    /// 宛先の国コード一覧（confidential のときのみ必須）
    #[serde(default)]
    pub recipients: Option<Vec<String>>,
    /// 本文
    pub context: String,
    /// 管理人発言かどうか（省略時 false）
    #[serde(default)]
    pub as_admin: bool,
}

///
/// 発言のリアルタイム配信クエリパラメータ構造体（Axum Query 抽出用）
///
#[derive(Debug, Deserialize)]
pub(crate) struct GetGameMessagesStreamQueryParams {
    pub after: Option<String>,
}
