// ============================================================================
// imports
// ============================================================================

// external crates
use chrono::NaiveDate;

// structs
use super::ApiErrorResponse;
use super::CreateGameCommand;
use super::CreateGameRequest;
use super::CreateGameResponse;
use super::Game;
use super::GameService;
use super::Regulation;

// enums
use super::CreateGameError;
use super::CreateGameRequestValidationError;
use super::Power;

// traits
use super::GameRepository;
use super::UserRepository;

// ============================================================================
// definitions
// ============================================================================

#[allow(dead_code)]
pub(crate) fn handle_create_game<U, G>(
    service: &GameService<U, G>,
    request: CreateGameRequest,
) -> Result<CreateGameResponse, CreateGameHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(CreateGameHandlerError::InvalidRequest)?;

    let regulation = parse_regulation(&request)?;
    let requested_power = parse_requested_power(request.requested_power.as_deref())?;
    let access_token = request.authorization.trim().trim_start_matches("Bearer ").trim().to_string();

    let result = service
        .create_game(CreateGameCommand {
            access_token,
            regulation,
            requested_power,
        })
        .map_err(CreateGameHandlerError::Service)?;

    Ok(CreateGameResponse {
        game_uuid: result.game.uuid,
        owner_user_uuid: result.owner_user_uuid,
        requested_power: result.requested_power.map(|power| power.to_string()),
    })
}

fn parse_regulation(request: &CreateGameRequest) -> Result<Regulation, CreateGameHandlerError> {
    let face_type = parse_enum(request.face_type, CreateGameRequestValidationError::InvalidFaceType)?;
    let progress_mode = parse_enum(request.progress_mode, CreateGameRequestValidationError::InvalidProgressMode)?;
    let duration_type = parse_enum(request.duration_type, CreateGameRequestValidationError::InvalidDurationType)?;
    let start_date = NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d")
        .map_err(|_| invalid_request(CreateGameRequestValidationError::InvalidStartDate))?;

    Regulation::new(face_type, progress_mode, duration_type, start_date, request.first_period_hour)
        .map_err(|_| invalid_request(CreateGameRequestValidationError::InvalidFirstPeriodHour))
}

fn parse_requested_power(value: Option<&str>) -> Result<Option<Power>, CreateGameHandlerError> {
    value
        .map(|code| Power::try_from(code).map_err(|_| invalid_request(CreateGameRequestValidationError::InvalidRequestedPower)))
        .transpose()
}

fn parse_enum<T>(value: i32, validation_error: CreateGameRequestValidationError) -> Result<T, CreateGameHandlerError>
where
    T: TryFrom<i32>,
{
    T::try_from(value).map_err(|_| invalid_request(validation_error))
}

fn invalid_request(validation_error: CreateGameRequestValidationError) -> CreateGameHandlerError {
    CreateGameHandlerError::InvalidRequest(validation_error)
}

#[derive(Debug)]
pub(crate) enum CreateGameHandlerError {
    InvalidRequest(CreateGameRequestValidationError),
    Service(CreateGameError),
}

#[allow(dead_code)]
impl CreateGameHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(CreateGameError::InvalidRequest(_)) => "invalid_request",
            Self::Service(CreateGameError::Unauthorized) => "unauthorized",
            Self::Service(CreateGameError::Repository(_)) => "repository_error",
            Self::Service(CreateGameError::Internal(_)) => "internal_error",
        }
    }

    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::InvalidRequest(CreateGameRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::MissingAccessToken) => "access token is required".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidFaceType) => "face_type is invalid".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidProgressMode) => "progress_mode is invalid".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidDurationType) => "duration_type is invalid".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidFirstPeriodHour) => {
                "first_period_hour must be between 0 and 23".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidStartDate) => {
                "start_date is invalid (expected YYYY-MM-DD)".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidRequestedPower) => {
                "requested_power is invalid".to_string()
            }
            Self::Service(error) => error.to_string(),
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
    use crate::repositories::NewGame;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        rows_by_token: Rc<RefCell<HashMap<String, UserRecord>>>,
    }

    impl InMemoryUserRepository {
        fn new(rows: Vec<UserRecord>) -> Self {
            let map = rows
                .into_iter()
                .map(|row| (row.access_token.clone(), row))
                .collect::<HashMap<_, _>>();
            Self {
                rows_by_token: Rc::new(RefCell::new(map)),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.rows_by_token.borrow().get(access_token).cloned())
        }

        fn insert(&self, _new_user: NewUser) -> Result<UserRecord, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn update_profile(&self, _id: UserId, _profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }
    }

    #[derive(Debug, Clone)]
    struct InMemoryGameRepository;

    impl GameRepository for InMemoryGameRepository {
        fn insert(&self, new_game: NewGame) -> Result<Game, RepositoryError> {
            Ok(new_game.game)
        }

        fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
            Ok(Vec::new())
        }

        fn find_by_uuid(&self, _game_uuid: uuid::Uuid) -> Result<Option<Game>, RepositoryError> {
            Ok(None)
        }

        fn find_progress_candidates(&self, _now: chrono::NaiveDateTime) -> Result<Vec<uuid::Uuid>, RepositoryError> {
            Ok(Vec::new())
        }

        fn try_claim_progression(
            &self,
            _game_uuid: uuid::Uuid,
            _lock_id: uuid::Uuid,
            _lock_until: chrono::NaiveDateTime,
            _now: chrono::NaiveDateTime,
        ) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        fn update_if_claimed(&self, _game: &Game, _lock_id: uuid::Uuid) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        fn release_progression_claim(&self, _game_uuid: uuid::Uuid, _lock_id: uuid::Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn update(&self, _game: &Game) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[test]
    fn handle_create_game_creates_owner_and_ready_phase() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "1001".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
        }]);
        let game_repository = InMemoryGameRepository;
        let service = GameService::new(user_repository, game_repository);

        let response = handle_create_game(
            &service,
            CreateGameRequest {
                authorization: "Bearer token-1".to_string(),
                face_type: 1,
                progress_mode: 1,
                duration_type: 1,
                start_date: "2026-04-19".to_string(),
                first_period_hour: 12,
                requested_power: Some("f".to_string()),
            },
        )
        .expect("create should succeed");

        assert_eq!(response.owner_user_uuid, owner_uuid);
        assert_eq!(response.requested_power.as_deref(), Some("France"));
    }

    #[test]
    fn handle_create_game_rejects_invalid_authorization() {
        let user_repository = InMemoryUserRepository::new(Vec::new());
        let game_repository = InMemoryGameRepository;
        let service = GameService::new(user_repository, game_repository);

        let error = handle_create_game(
            &service,
            CreateGameRequest {
                authorization: "token-1".to_string(),
                face_type: 1,
                progress_mode: 1,
                duration_type: 1,
                start_date: "2026-04-19".to_string(),
                first_period_hour: 12,
                requested_power: None,
            },
        )
        .expect_err("create should fail");

        assert_eq!(error.code(), "invalid_request");
    }
}
