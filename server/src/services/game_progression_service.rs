// ============================================================================
// imports
// ============================================================================

use chrono::Utc;
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;

use super::GameProgressionError;
use super::GameRepository;
use super::GameStatus;
use super::PhaseContext;
use super::UserRepository;

// ============================================================================
// definitions
// ============================================================================

///
/// 卓進行サービスの構造体
///
pub(crate) struct GameProgressionService<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    user_repository: U,
    game_repository: G,
}

type StartedSeasonMessage = (uuid::Uuid, String, String);
type SoloMessage = (uuid::Uuid, super::Power);
type DrawMessage = uuid::Uuid;
type ClosedMessage = uuid::Uuid;
type ProgressGamesResult = (
    Vec<uuid::Uuid>,
    Vec<StartedSeasonMessage>,
    Vec<SoloMessage>,
    Vec<DrawMessage>,
    Vec<ClosedMessage>,
);
type ProgressGameResult = (bool, Option<(String, String)>, Option<super::Power>, bool, bool);
const SOLO_SUPPLY_CENTER_THRESHOLD: usize = 18;

/// 卓進行サービスの構造体の実装
impl<U, G> GameProgressionService<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    ///
    /// new 関数
    ///
    pub(crate) fn new(user_repository: U, game_repository: G) -> Self {
        Self {
            user_repository,
            game_repository,
        }
    }

    ///
    /// 卓進行処理を実行する
    ///
    /// Closed・Aborted 以外の全 Game に対してフェイズ進行を試みる。
    /// next_update_at が現在時刻より過去の場合のみ進行処理を実行する。
    pub(crate) fn progress_games(&self) -> Result<ProgressGamesResult, GameProgressionError> {
        let now = Utc::now().naive_utc();
        let mut aborted_game_uuids = Vec::new();
        let mut started_seasons = Vec::new();
        let mut solo_games = Vec::new();
        let mut draw_games = Vec::new();
        let mut closed_games = Vec::new();

        let candidate_game_uuids = self
            .game_repository
            .find_progress_candidates(now)
            .map_err(GameProgressionError::Repository)?;

        for game_uuid in candidate_game_uuids {
            let (is_aborted, started_season, solo_power, is_draw, is_closed) = self.progress_game(game_uuid, now)?;

            if is_aborted {
                aborted_game_uuids.push(game_uuid);
            }

            if let Some((turn, season)) = started_season {
                started_seasons.push((game_uuid, turn, season));
            }

            if let Some(power) = solo_power {
                solo_games.push((game_uuid, power));
            }

            if is_draw {
                draw_games.push(game_uuid);
            }

            if is_closed {
                closed_games.push(game_uuid);
            }
        }

        Ok((aborted_game_uuids, started_seasons, solo_games, draw_games, closed_games))
    }

    /// 卓別の進行処理を実行する
    ///
    /// next_update_at が None または未来の場合は何もしない。
    fn progress_game(
        &self,
        game_uuid: uuid::Uuid,
        now: chrono::NaiveDateTime,
    ) -> Result<ProgressGameResult, GameProgressionError> {
        let Some(mut game) = self
            .game_repository
            .find_by_uuid(game_uuid)
            .map_err(GameProgressionError::Repository)?
        else {
            return Ok((false, None, None, false, false));
        };

        let Some(previous_next_update) = game.next_update_at else {
            return Ok((false, None, None, false, false));
        };

        if game.status == GameStatus::Closed || game.status == GameStatus::Aborted || previous_next_update > now {
            return Ok((false, None, None, false, false));
        }

        let latest_phase = game.phases.pop().expect("game should have at least one phase");

        // 募集不成立チェック: Ready フェイズ到達時点でプレイヤーが 7 人未満なら中止
        if matches!(latest_phase.kind, crate::domain::PhaseKind::Ready(_))
            && game.status != GameStatus::InProgress
            && game.players.iter().filter(|p| p.power.is_some()).count() < crate::domain::Power::iter().count()
        {
            game.phases.push(latest_phase);
            game.status = GameStatus::Aborted;
            game.next_update_at = None;
            self.game_repository.update(&game).map_err(GameProgressionError::Repository)?;
            return Ok((true, None, None, false, false));
        }

        // フェイズ進行処理
        let mut context = PhaseContext::new();
        let owner_accepting_draw_on_main = Self::is_owner_accepting_draw(&game) && Self::is_draw_applicable_phase(&latest_phase);
        let latest_phase_is_debrief = matches!(latest_phase.kind, crate::domain::PhaseKind::Debrief(_));
        if owner_accepting_draw_on_main {
            context.set_draw();
        }
        self.remove_idle_powers(&game, &mut context, now)?;
        latest_phase.close(&mut context);

        // 卓終了判定
        let is_finished = context.is_finished();
        let is_debrief_closed = latest_phase_is_debrief && !is_finished;
        let new_next_update = if is_debrief_closed {
            // Debrief フェイズが終了 → 卓閉鎖
            None
        } else if is_finished {
            // Solo/Draw → Debrief フェイズの next_update を計算
            let debrief_phase = context.phases().back().expect("finished game should have debrief phase");
            Some(game.calculate_next_update(previous_next_update, debrief_phase))
        } else {
            let next_phase = context.phases().back().expect("in-progress game should have next phase");
            Some(game.calculate_next_update(previous_next_update, next_phase))
        };

        // PhaseContext の結果を Game に反映
        for phase in context.phases().iter() {
            game.phases.push(phase.clone());
        }
        game.is_draw = context.is_draw();
        game.is_solo = context.is_solo();
        game.status = if is_debrief_closed {
            GameStatus::Closed
        } else if is_finished {
            GameStatus::Finished
        } else {
            GameStatus::InProgress
        };
        game.next_update_at = new_next_update;

        // 卓永続化処理
        self.game_repository.update(&game).map_err(GameProgressionError::Repository)?;

        let started_season = if matches!(
            game.phases.last().map(|phase| phase.kind),
            Some(crate::domain::PhaseKind::SpringMain(_)) | Some(crate::domain::PhaseKind::FallMain(_))
        ) {
            game.current_season_label().map(|season| (game.current_turn(), season))
        } else {
            None
        };

        let solo_power = if game.is_solo {
            game.phases.last().and_then(|phase| {
                super::Power::iter().find(|power| phase.count_supply_centers(power) >= SOLO_SUPPLY_CENTER_THRESHOLD)
            })
        } else {
            None
        };

        let is_draw = game.is_draw;

        Ok((false, started_season, solo_power, is_draw, is_debrief_closed))
    }

    /// 卓主が和平終了を承認しているかどうかを返却する
    fn is_owner_accepting_draw(game: &super::Game) -> bool {
        game.players
            .iter()
            .find(|player| player.is_owner)
            .is_some_and(|player| player.is_accepting_draw)
    }

    /// 和平終了が適用されるフェイズかどうかを返却する
    fn is_draw_applicable_phase(phase: &super::Phase) -> bool {
        matches!(
            phase.kind,
            crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_)
        )
    }

    /// 無政府国を活性国リストから除外する
    fn remove_idle_powers(
        &self,
        game: &super::Game,
        context: &mut PhaseContext,
        now: chrono::NaiveDateTime,
    ) -> Result<(), GameProgressionError> {
        let threshold = Self::idle_threshold(game, now);

        for player in game.players.iter().filter(|player| player.power.is_some()) {
            let power = player.power.expect("filtered as Some");

            let user = self
                .user_repository
                .find_by_uuid(player.user_uuid)
                .map_err(GameProgressionError::Repository)?;

            if Self::is_idle_or_missing(user, threshold) {
                context.remove_power(&power);
            }
        }

        Ok(())
    }

    /// 卓主無政府化卓の和平終了処理を実行する
    pub(crate) fn mark_idle_owners_accepting_draw(&self) -> Result<(), GameProgressionError> {
        let now = Utc::now().naive_utc();

        let games = self
            .game_repository
            .find_all_active()
            .map_err(GameProgressionError::Repository)?;

        for mut game in games {
            if self.mark_owner_accepting_draw_if_idle(&mut game, now)? {
                self.game_repository.update(&game).map_err(GameProgressionError::Repository)?;
            }
        }

        Ok(())
    }

    /// 指定卓の卓主が現時刻基準で無政府状態かどうかを返す
    pub(crate) fn is_owner_idle_for_game(
        &self,
        game_uuid: uuid::Uuid,
        now: chrono::NaiveDateTime,
    ) -> Result<bool, GameProgressionError> {
        let Some(game) = self
            .game_repository
            .find_by_uuid(game_uuid)
            .map_err(GameProgressionError::Repository)?
        else {
            return Ok(false);
        };

        if game.status != super::GameStatus::InProgress {
            return Ok(false);
        }

        let Some(owner) = game.players.iter().find(|p| p.is_owner) else {
            return Ok(false);
        };

        let threshold = Self::idle_threshold(&game, now);
        let user = self
            .user_repository
            .find_by_uuid(owner.user_uuid)
            .map_err(GameProgressionError::Repository)?;

        Ok(Self::is_idle_or_missing(user, threshold))
    }

    /// 卓別卓主無政府化卓に和平終了フラグを立てる
    fn mark_owner_accepting_draw_if_idle(
        &self,
        game: &mut super::Game,
        now: chrono::NaiveDateTime,
    ) -> Result<bool, GameProgressionError> {
        // 進行中の卓のみを対象とする
        if game.status != super::GameStatus::InProgress {
            return Ok(false);
        }

        let Some(owner) = game.players.iter().find(|p| p.is_owner) else {
            return Ok(false);
        };

        // すでにフラグが立っている場合はスキップ（冪等）
        if owner.is_accepting_draw {
            return Ok(false);
        }

        let threshold = Self::idle_threshold(game, now);

        let user = self
            .user_repository
            .find_by_uuid(owner.user_uuid)
            .map_err(GameProgressionError::Repository)?;

        if !Self::is_idle_or_missing(user, threshold) {
            return Ok(false);
        }

        game.players
            .iter_mut()
            .find(|p| p.is_owner)
            .expect("owner exists")
            .is_accepting_draw = true;

        Ok(true)
    }

    /// 無政府判定閾値を算出する
    fn idle_threshold(game: &super::Game, now: chrono::NaiveDateTime) -> chrono::NaiveDateTime {
        let idle_limit = chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes()));
        now - idle_limit
    }

    /// 無政府ユーザーを判定する
    fn is_idle_or_missing(user: Option<super::UserRecord>, threshold: chrono::NaiveDateTime) -> bool {
        match user {
            Some(user) => user.last_access_at.naive_utc() <= threshold,
            None => true,
        }
    }

    /// プレイヤーに担当国を割り当てる
    ///
    /// 割り当てルール:
    /// - 卓主には無条件で希望国が割り当てられる。
    /// - 卓主と希望が被った参加者は抽選に外れた扱いで最後に回される。
    /// - 希望者が一人しかいない国はその一人に割り当てられる。
    /// - 希望者が複数いる国は抽選で希望者の一人に割り当てられる。
    /// - 残った国は残った参加者にランダムで割り当てられる。
    pub(crate) fn assign_powers(players: &mut [super::Player]) {
        use std::collections::HashMap;
        use std::collections::HashSet;
        use strum::IntoEnumIterator;

        let mut rng = rand::rng();

        // Step 1: 卓主の希望国を無条件で割り当て
        let owner_requested_power = players.iter().find(|p| p.is_owner).and_then(|p| p.requested_power);

        if let Some(owner_power) = owner_requested_power {
            players
                .iter_mut()
                .filter(|p| p.is_owner)
                .for_each(|p| p.power = Some(owner_power));
        }

        // 割り当て済みパワーを除いた残りパワーリスト
        let assigned_set: HashSet<super::Power> = players.iter().filter_map(|p| p.power).collect();
        let mut remaining_powers: Vec<super::Power> = super::Power::iter().filter(|p| !assigned_set.contains(p)).collect();

        // Step 2: 卓主と希望が被った参加者を抽選落ちとして識別
        let mut lottery_losers: Vec<uuid::Uuid> = Vec::new();
        let mut lottery_loser_set: HashSet<uuid::Uuid> = HashSet::new();

        if let Some(owner_power) = owner_requested_power {
            for player in players.iter().filter(|p| !p.is_owner && p.power.is_none()) {
                if player.requested_power == Some(owner_power) {
                    lottery_losers.push(player.user_uuid);
                    lottery_loser_set.insert(player.user_uuid);
                }
            }
        }

        // Step 3: 希望国ごとに参加者をグループ化（割り当て済み・抽選落ちを除く）
        let mut requests: HashMap<super::Power, Vec<uuid::Uuid>> = HashMap::new();
        let mut no_preference: Vec<uuid::Uuid> = Vec::new();

        for player in players.iter() {
            if player.power.is_some() {
                continue;
            }
            if lottery_loser_set.contains(&player.user_uuid) {
                continue;
            }
            match player.requested_power {
                Some(req) => requests.entry(req).or_default().push(player.user_uuid),
                None => no_preference.push(player.user_uuid),
            }
        }

        // Step 4: 希望国の割り当て
        for (power, mut applicants) in requests {
            if applicants.len() == 1 {
                // 希望者 1 人 → 割り当て
                let uuid = applicants[0];
                players
                    .iter_mut()
                    .filter(|p| p.user_uuid == uuid)
                    .for_each(|p| p.power = Some(power));
                remaining_powers.retain(|&rp| rp != power);
            } else {
                // 複数希望者 → 抽選
                applicants.shuffle(&mut rng);
                let winner_uuid = applicants[0];
                players
                    .iter_mut()
                    .filter(|p| p.user_uuid == winner_uuid)
                    .for_each(|p| p.power = Some(power));
                remaining_powers.retain(|&rp| rp != power);
                for loser_uuid in applicants.into_iter().skip(1) {
                    lottery_losers.push(loser_uuid);
                }
            }
        }

        // Step 5: 残りのパワーをシャッフルし、無希望→抽選落ち（卓主と被った参加者を最後に）の順に割り当て
        remaining_powers.shuffle(&mut rng);
        let all_remaining: Vec<uuid::Uuid> = no_preference.into_iter().chain(lottery_losers).collect();

        for (user_uuid, power) in all_remaining.into_iter().zip(remaining_powers) {
            players
                .iter_mut()
                .filter(|p| p.user_uuid == user_uuid)
                .for_each(|p| p.power = Some(power));
        }
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;
    use crate::domain::DurationType;
    use crate::domain::FaceType;
    use crate::domain::Game;
    use crate::domain::Phase;
    use crate::domain::Player;
    use crate::domain::Power;
    use crate::domain::ProgressMode;
    use crate::domain::Regulation;
    use crate::repositories::NewGame;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct InMemoryGameRepository {
        active_games: Rc<RefCell<Vec<Game>>>,
        updated_games: Rc<RefCell<Vec<Game>>>,
    }

    impl InMemoryGameRepository {
        fn new(active_games: Vec<Game>) -> Self {
            Self {
                active_games: Rc::new(RefCell::new(active_games)),
                updated_games: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn updated_len(&self) -> usize {
            self.updated_games.borrow().len()
        }

        fn updated_first(&self) -> Option<Game> {
            self.updated_games.borrow().first().cloned()
        }
    }

    impl GameRepository for InMemoryGameRepository {
        fn insert(&self, _new_game: NewGame) -> Result<Game, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
            Ok(self.active_games.borrow().clone())
        }

        fn find_by_uuid(&self, game_uuid: uuid::Uuid) -> Result<Option<Game>, RepositoryError> {
            Ok(self.active_games.borrow().iter().find(|game| game.uuid == game_uuid).cloned())
        }

        fn find_progress_candidates(&self, now: chrono::NaiveDateTime) -> Result<Vec<uuid::Uuid>, RepositoryError> {
            Ok(self
                .active_games
                .borrow()
                .iter()
                .filter(|game| game.status != GameStatus::Closed && game.status != GameStatus::Aborted)
                .filter(|game| game.next_update_at.is_some_and(|next_update| next_update <= now))
                .map(|game| game.uuid)
                .collect())
        }

        fn exists_active_game_for_user(&self, _user_uuid: uuid::Uuid) -> Result<bool, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn update(&self, game: &Game) -> Result<(), RepositoryError> {
            self.updated_games.borrow_mut().push(game.clone());
            Ok(())
        }

        fn add_player(
            &self,
            _game_uuid: uuid::Uuid,
            _user_uuid: uuid::Uuid,
            _requested_power: Option<Power>,
        ) -> Result<(), RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn assign_game_number(&self, _game_uuid: uuid::Uuid) -> Result<i32, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }
    }

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        rows_by_uuid: Rc<RefCell<HashMap<uuid::Uuid, UserRecord>>>,
    }

    impl InMemoryUserRepository {
        fn new(rows: Vec<UserRecord>) -> Self {
            let map = rows.into_iter().map(|row| (row.uuid, row)).collect();
            Self {
                rows_by_uuid: Rc::new(RefCell::new(map)),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_uuid(&self, user_uuid: uuid::Uuid) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.rows_by_uuid.borrow().get(&user_uuid).cloned())
        }

        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self
                .rows_by_uuid
                .borrow()
                .values()
                .find(|row| row.access_token == access_token)
                .cloned())
        }

        fn update_last_access_at_by_access_token(
            &self,
            access_token: &str,
            last_access_at: chrono::DateTime<chrono::Utc>,
        ) -> Result<bool, RepositoryError> {
            let mut rows = self.rows_by_uuid.borrow_mut();
            let Some(row) = rows.values_mut().find(|row| row.access_token == access_token) else {
                return Ok(false);
            };

            row.last_access_at = last_access_at;
            Ok(true)
        }

        fn insert(&self, _new_user: NewUser) -> Result<UserRecord, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn update_profile(&self, _id: UserId, _profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }
    }

    fn sample_regulation() -> Regulation {
        Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation")
    }

    fn sample_game(next_update: Option<chrono::NaiveDateTime>) -> Game {
        sample_game_with_regulation(next_update, sample_regulation())
    }

    fn sample_game_with_regulation(next_update: Option<chrono::NaiveDateTime>, regulation: Regulation) -> Game {
        let owner_uuid = uuid::Uuid::now_v7();
        Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: owner_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: next_update,
        }
    }

    /// 全 7 プレイヤーが揃った状態のゲームを生成するヘルパー
    fn sample_full_game(next_update: Option<chrono::NaiveDateTime>) -> Game {
        use strum::IntoEnumIterator;
        let mut game = sample_game(next_update);
        for power in Power::iter().filter(|p| *p != Power::France) {
            game.players.push(Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: Some(power),
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(power),
            });
        }
        game
    }

    /// 指定した regulation で全 7 プレイヤーが揃った状態のゲームを生成するヘルパー
    fn sample_full_game_with_regulation(next_update: Option<chrono::NaiveDateTime>, regulation: Regulation) -> Game {
        use strum::IntoEnumIterator;
        let mut game = sample_game_with_regulation(next_update, regulation);
        for power in Power::iter().filter(|p| *p != Power::France) {
            game.players.push(Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: Some(power),
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(power),
            });
        }
        game
    }

    /// `sample_full_game` の全プレイヤーに対応する UserRecord 一覧を生成するヘルパー
    fn users_for_full_game(game: &Game, last_access_at: chrono::DateTime<chrono::Utc>) -> Vec<UserRecord> {
        game.players
            .iter()
            .map(|p| UserRecord {
                id: 1,
                uuid: p.user_uuid,
                discord_user_id: format!("discord-{}", p.user_uuid),
                username: format!("user-{}", p.user_uuid),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: format!("token-{}", p.user_uuid),
                last_access_at,
            })
            .collect()
    }

    fn user_for(game: &Game, power: Power, last_access_at: chrono::DateTime<chrono::Utc>) -> UserRecord {
        let user_uuid = game
            .players
            .iter()
            .find(|player| player.power == Some(power))
            .map(|player| player.user_uuid)
            .expect("player for power should exist");

        UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: format!("discord-{}", power.symbol()),
            username: format!("{}-user", power.symbol()),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: format!("token-{}", power.symbol()),
            last_access_at,
        }
    }

    #[test]
    fn progress_games_skips_when_next_update_is_none() {
        let game = sample_game(None);
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn progress_games_skips_when_next_update_is_in_future() {
        let future = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(5);
        let game = sample_game(Some(future));
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn progress_games_updates_when_next_update_is_in_past() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");
        let game = sample_full_game(Some(past));
        let users = InMemoryUserRepository::new(users_for_full_game(&game, chrono::Utc::now()));
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(updated.status, GameStatus::InProgress);
        assert_eq!(
            updated.next_update_at,
            Some(past_date.and_hms_opt(15, 5, 0).expect("valid datetime"))
        );
    }

    #[test]
    fn progress_games_sets_next_main_phase_to_next_day_first_period_hour_for_scheduled_normal() {
        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Normal,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            21,
        )
        .expect("valid regulation");
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");
        let game = sample_full_game_with_regulation(Some(past), regulation);
        let users = InMemoryUserRepository::new(users_for_full_game(&game, chrono::Utc::now()));
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(
            updated.next_update_at,
            Some(
                past_date
                    .succ_opt()
                    .expect("next day should exist")
                    .and_hms_opt(12, 0, 0)
                    .expect("valid datetime")
            )
        );
    }

    #[test]
    fn progress_games_skips_retreat_wait_for_idle_power() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");

        let mut game = sample_game(Some(past));
        let france_user_uuid = uuid::Uuid::now_v7();
        let germany_user_uuid = uuid::Uuid::now_v7();
        game.players = vec![
            Player {
                user_uuid: france_user_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            },
            Player {
                user_uuid: germany_user_uuid,
                power: Some(Power::Germany),
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(Power::Germany),
            },
        ];
        let mut spring_main = Phase::new_spring_main(1900, 0);
        spring_main.units = vec![
            crate::domain::Unit::new_army(
                Power::Germany,
                crate::domain::Province::from_code("ber").expect("valid province"),
            )
            .set_dislodged_from(Some(crate::domain::Province::from_code("kie").expect("valid province"))),
        ];
        spring_main.territories = vec![crate::domain::Territory::new(Power::Germany, "ber")];
        game.phases = vec![spring_main];
        game.status = GameStatus::InProgress;

        let users = InMemoryUserRepository::new(vec![
            UserRecord {
                id: 1,
                uuid: france_user_uuid,
                discord_user_id: "discord-f".to_string(),
                username: "f-user".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-f".to_string(),
                last_access_at: chrono::Utc::now(),
            },
            UserRecord {
                id: 2,
                uuid: germany_user_uuid,
                discord_user_id: "discord-g".to_string(),
                username: "g-user".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-g".to_string(),
                last_access_at: chrono::Utc::now()
                    - chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes() + 1)),
            },
        ]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        let updated = repository.updated_first().expect("updated game should exist");
        assert!(matches!(
            updated.phases.last().expect("phase should exist").kind,
            crate::domain::PhaseKind::FallMain(_)
        ));
    }

    #[test]
    fn progress_games_finishes_with_draw_when_owner_accepts_draw() {
        let past = chrono::Utc::now().naive_utc() - chrono::Duration::minutes(1);

        let mut game = sample_game(Some(past));
        game.phases = vec![Phase::new_spring_main(1900, 0)];
        game.status = GameStatus::InProgress;
        game.players[0].is_accepting_draw = true;

        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert!(updated.is_draw);
        assert!(!updated.is_solo);
        assert_eq!(updated.status, GameStatus::Finished);
        assert!(updated.next_update_at.is_some());
        assert!(matches!(
            updated.phases.last().expect("phase should exist").kind,
            crate::domain::PhaseKind::Debrief(_)
        ));
    }

    #[test]
    fn progress_games_does_not_finish_with_draw_when_owner_accepts_in_non_main_phase() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");

        let mut game = sample_game(Some(past));
        game.phases = vec![Phase::new_ready()];
        game.status = GameStatus::InProgress;
        game.players[0].is_accepting_draw = true;

        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert!(!updated.is_draw);
        assert_eq!(updated.status, GameStatus::InProgress);
        assert!(matches!(
            updated.phases.last().expect("phase should exist").kind,
            crate::domain::PhaseKind::SpringMain(_)
        ));
    }

    #[test]
    fn mark_idle_owners_accepting_draw_sets_flag_for_idle_owner() {
        let mut game = sample_game(None);
        let idle_last_access =
            chrono::Utc::now() - chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes() + 1));
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, idle_last_access)]);
        game.status = GameStatus::InProgress;
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.mark_idle_owners_accepting_draw().expect("should succeed");

        let updated = repository.updated_first().expect("game should be updated");
        assert!(updated.players.iter().find(|p| p.is_owner).unwrap().is_accepting_draw);
    }

    #[test]
    fn mark_idle_owners_accepting_draw_does_not_set_flag_for_active_owner() {
        let mut game = sample_game(None);
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        game.status = GameStatus::InProgress;
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.mark_idle_owners_accepting_draw().expect("should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn mark_idle_owners_accepting_draw_skips_when_flag_already_set() {
        let mut game = sample_game(None);
        let idle_last_access =
            chrono::Utc::now() - chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes() + 1));
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, idle_last_access)]);
        game.status = GameStatus::InProgress;
        game.players[0].is_accepting_draw = true;
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.mark_idle_owners_accepting_draw().expect("should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn mark_idle_owners_accepting_draw_skips_non_in_progress_games() {
        let game = sample_game(None);
        let idle_last_access =
            chrono::Utc::now() - chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes() + 1));
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, idle_last_access)]);
        // Preparing のままにする（InProgress にしない）
        assert_eq!(game.status, GameStatus::Preparing);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.mark_idle_owners_accepting_draw().expect("should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn progress_games_aborts_when_ready_phase_has_fewer_than_7_players() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");

        // Ready フェイズ・プレイヤー 1 人（7 人未満）
        let game = sample_game(Some(past));
        assert_eq!(game.players.iter().filter(|p| p.power.is_some()).count(), 1);

        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(updated.status, GameStatus::Aborted);
        assert!(updated.next_update_at.is_none());
        // Ready フェイズが履歴に残っている
        assert!(matches!(
            updated.phases.last().expect("phase should exist").kind,
            crate::domain::PhaseKind::Ready(_)
        ));
    }

    #[test]
    fn progress_games_does_not_abort_when_ready_phase_has_7_players() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");

        let game = sample_full_game(Some(past));
        let users = InMemoryUserRepository::new(users_for_full_game(&game, chrono::Utc::now()));
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(updated.status, GameStatus::InProgress);
    }

    // ============================================================
    // assign_powers のテスト
    // ============================================================

    /// 全 7 人（power: None）のプレイヤー一覧を生成するヘルパー
    /// 希望国はそれぞれ一意に設定する
    fn sample_players_unassigned() -> Vec<Player> {
        use strum::IntoEnumIterator;
        let powers: Vec<Power> = Power::iter().collect();
        powers
            .iter()
            .enumerate()
            .map(|(i, &p)| Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: i == 0, // 最初のプレイヤーが卓主
                requested_power: Some(p),
            })
            .collect()
    }

    #[test]
    fn assign_powers_owner_gets_requested_power() {
        let mut players = sample_players_unassigned();
        let owner_requested = players.iter().find(|p| p.is_owner).unwrap().requested_power;

        GameProgressionService::<InMemoryUserRepository, InMemoryGameRepository>::assign_powers(&mut players);

        let owner = players.iter().find(|p| p.is_owner).unwrap();
        assert_eq!(owner.power, owner_requested, "卓主は希望国を取得するべき");
    }

    #[test]
    fn assign_powers_all_players_receive_a_power() {
        let mut players = sample_players_unassigned();

        GameProgressionService::<InMemoryUserRepository, InMemoryGameRepository>::assign_powers(&mut players);

        assert!(
            players.iter().all(|p| p.power.is_some()),
            "全プレイヤーに担当国が割り当てられるべき"
        );
    }

    #[test]
    fn assign_powers_no_duplicate_powers() {
        use std::collections::HashSet;
        let mut players = sample_players_unassigned();

        GameProgressionService::<InMemoryUserRepository, InMemoryGameRepository>::assign_powers(&mut players);

        let assigned: Vec<Power> = players.iter().filter_map(|p| p.power).collect();
        let unique: HashSet<Power> = assigned.iter().cloned().collect();
        assert_eq!(assigned.len(), unique.len(), "担当国は重複しないべき");
    }

    #[test]
    fn assign_powers_single_applicant_gets_requested_power() {
        use strum::IntoEnumIterator;
        // 卓主は Austria を希望、残りは順番に異なる国を希望（全員希望が一意）
        let powers: Vec<Power> = Power::iter().collect();
        let mut players: Vec<Player> = powers
            .iter()
            .enumerate()
            .map(|(i, &p)| Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: i == 0,
                requested_power: Some(p),
            })
            .collect();

        GameProgressionService::<InMemoryUserRepository, InMemoryGameRepository>::assign_powers(&mut players);

        // 全員希望が一意なので全員自分の希望国を得るはず
        for player in &players {
            assert_eq!(
                player.power, player.requested_power,
                "希望者が 1 人の国はその人に割り当てられるべき"
            );
        }
    }

    #[test]
    fn assign_powers_conflict_with_owner_player_gets_some_power() {
        use strum::IntoEnumIterator;
        // 卓主は Austria を希望、参加者 1 人も Austria を希望（衝突）
        let powers: Vec<Power> = Power::iter().collect();
        let owner_uuid = uuid::Uuid::now_v7();
        let conflicting_uuid = uuid::Uuid::now_v7();
        let mut players: Vec<Player> = vec![
            Player {
                user_uuid: owner_uuid,
                power: None,
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(powers[0]), // Austria
            },
            Player {
                user_uuid: conflicting_uuid,
                power: None,
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(powers[0]), // Austria（衝突）
            },
        ];
        // 残り 5 人はそれぞれ別の国を希望
        for &p in powers.iter().skip(1).take(5) {
            players.push(Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(p),
            });
        }

        GameProgressionService::<InMemoryUserRepository, InMemoryGameRepository>::assign_powers(&mut players);

        // 卓主は Austria を取得
        let owner = players.iter().find(|p| p.user_uuid == owner_uuid).unwrap();
        assert_eq!(owner.power, Some(powers[0]), "卓主は Austria を取得するべき");

        // 衝突した参加者は Austria 以外の何らかの国を取得
        let conflicting = players.iter().find(|p| p.user_uuid == conflicting_uuid).unwrap();
        assert!(conflicting.power.is_some(), "衝突した参加者も担当国を取得するべき");
        assert_ne!(
            conflicting.power,
            Some(powers[0]),
            "衝突した参加者は Austria を取得しないべき"
        );

        // 全員に重複なく割り当て
        use std::collections::HashSet;
        let assigned: HashSet<Power> = players.iter().filter_map(|p| p.power).collect();
        assert_eq!(assigned.len(), 7);
    }

    #[test]
    fn assign_powers_multiple_applicants_one_wins() {
        use strum::IntoEnumIterator;
        // 卓主は Austria を希望、他に England を希望する参加者が 3 人
        let powers: Vec<Power> = Power::iter().collect();
        let mut players: Vec<Player> = vec![Player {
            user_uuid: uuid::Uuid::now_v7(),
            power: None,
            is_accepting_draw: false,
            is_owner: true,
            requested_power: Some(powers[0]), // Austria
        }];
        // England を 3 人で希望
        let england = powers[1];
        for _ in 0..3 {
            players.push(Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(england),
            });
        }
        // 残り 3 人はそれぞれ別の国を希望
        for &p in powers.iter().skip(2).take(3) {
            players.push(Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(p),
            });
        }

        GameProgressionService::<InMemoryUserRepository, InMemoryGameRepository>::assign_powers(&mut players);

        // England を取得したプレイヤーはちょうど 1 人
        let england_count = players.iter().filter(|p| p.power == Some(england)).count();
        assert_eq!(england_count, 1, "複数希望の国を取得するのはちょうど 1 人であるべき");

        // 全員に担当国が割り当てられていて重複なし
        use std::collections::HashSet;
        let assigned: HashSet<Power> = players.iter().filter_map(|p| p.power).collect();
        assert_eq!(assigned.len(), 7);
    }

    #[test]
    fn assign_powers_owner_without_requested_power_gets_some_power() {
        use strum::IntoEnumIterator;
        let powers: Vec<Power> = Power::iter().collect();
        let mut players: Vec<Player> = vec![Player {
            user_uuid: uuid::Uuid::now_v7(),
            power: None,
            is_accepting_draw: false,
            is_owner: true,
            requested_power: None, // 希望なし
        }];
        for &p in powers.iter() {
            players.push(Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(p),
            });
        }
        // 8 人になってしまうので 1 人除く
        players.pop();

        GameProgressionService::<InMemoryUserRepository, InMemoryGameRepository>::assign_powers(&mut players);

        let owner = players.iter().find(|p| p.is_owner).unwrap();
        assert!(owner.power.is_some(), "希望なしの卓主も担当国を取得するべき");

        use std::collections::HashSet;
        let assigned: HashSet<Power> = players.iter().filter_map(|p| p.power).collect();
        assert_eq!(assigned.len(), 7);
    }
}
