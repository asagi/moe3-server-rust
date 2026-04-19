use chrono::NaiveDate;

use crate::api::requests::CreateGameRequest;
use crate::api::requests::CreateGameRequestValidationError;
use crate::api::responses::ApiErrorResponse;
use crate::api::responses::CreateGameResponse;
use crate::domain::DurationType;
use crate::domain::FaceType;
use crate::domain::Power;
use crate::domain::ProgressMode;
use crate::domain::Regulation;
use crate::repositories::GameRepository;
use crate::repositories::UserRepository;
use crate::services::CreateGameCommand;
use crate::services::CreateGameError;
use crate::services::GameService;

pub(crate) fn handle_create_game<U, G>(
    service: &GameService<U, G>,
    request: CreateGameRequest,
) -> Result<CreateGameResponse, CreateGameHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(CreateGameHandlerError::InvalidRequest)?;

    let face_type = FaceType::try_from(request.face_type)
        .map_err(|_| CreateGameHandlerError::InvalidRequest(CreateGameRequestValidationError::InvalidFaceType))?;
    let progress_mode = ProgressMode::try_from(request.progress_mode)
        .map_err(|_| CreateGameHandlerError::InvalidRequest(CreateGameRequestValidationError::InvalidProgressMode))?;
    let duration_type = DurationType::try_from(request.duration_type)
        .map_err(|_| CreateGameHandlerError::InvalidRequest(CreateGameRequestValidationError::InvalidDurationType))?;

    let start_date = NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d")
        .map_err(|_| CreateGameHandlerError::InvalidRequest(CreateGameRequestValidationError::InvalidStartDate))?;

    let requested_power = match request.requested_power {
        Some(value) => Some(power_from_i32(value).ok_or(CreateGameHandlerError::InvalidRequest(
            CreateGameRequestValidationError::InvalidRequestedPower,
        ))?),
        None => None,
    };

    let regulation = Regulation::new(face_type, progress_mode, duration_type, start_date, request.first_period_hour)
        .map_err(|_| CreateGameHandlerError::InvalidRequest(CreateGameRequestValidationError::InvalidFirstPeriodHour))?;

    let access_token = request.authorization.trim().trim_start_matches("Bearer ").trim().to_string();

    let result = service
        .create_game(CreateGameCommand {
            access_token,
            regulation,
            requested_power,
        })
        .map_err(CreateGameHandlerError::Service)?;

    let owner = result
        .game
        .players
        .iter()
        .find(|player| player.is_owner)
        .ok_or(CreateGameHandlerError::Service(CreateGameError::Repository(
            crate::repositories::RepositoryError::Unavailable("owner player is missing".to_string()),
        )))?;

    Ok(CreateGameResponse {
        game_uuid: result.game.uuid,
        owner_user_uuid: owner.user_uuid,
        requested_power: owner.requested_power.map(|power| power.to_string()),
    })
}

fn power_from_i32(value: i32) -> Option<Power> {
    match value {
        1 => Some(Power::Austria),
        2 => Some(Power::England),
        3 => Some(Power::France),
        4 => Some(Power::Germany),
        5 => Some(Power::Italy),
        6 => Some(Power::Russia),
        7 => Some(Power::Turkey),
        _ => None,
    }
}

#[derive(Debug)]
#[cfg_attr(not(test), allow(dead_code))]
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
                "authorization must start with Bearer".to_string()
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
        fn insert(&self, new_game: NewGame) -> Result<crate::domain::Game, RepositoryError> {
            Ok(new_game.game)
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
                requested_power: Some(3),
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
