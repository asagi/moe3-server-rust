use crate::api::requests::LoginRequest;
use crate::api::requests::RequestValidationError;
use crate::api::responses::ApiErrorResponse;
use crate::api::responses::LoginResponse;
use crate::api::responses::LoginUserResponse;
use crate::services::AuthError;
use crate::services::DiscordClientError;
use crate::services::LoginCommand;
use crate::services::LoginService;

pub(crate) fn handle_login<S>(service: &S, request: LoginRequest) -> Result<LoginResponse, LoginHandlerError>
where
    S: LoginService,
{
    request.validate().map_err(LoginHandlerError::InvalidRequest)?;

    let result = service
        .login(LoginCommand {
            discord_access_token: request.discord_access_token,
        })
        .map_err(LoginHandlerError::Service)?;

    Ok(LoginResponse {
        access_token: result.access_token,
        user: LoginUserResponse {
            discord_user_id: result.user.discord_user_id,
            display_name: result.user.display_name,
            avatar_hash: result.user.avatar_hash,
            avatar_url: result.user.avatar_url,
        },
    })
}

#[derive(Debug)]
pub(crate) enum LoginHandlerError {
    InvalidRequest(RequestValidationError),
    Service(AuthError),
}

impl LoginHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(AuthError::InvalidRequest(_)) => "invalid_request",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unauthorized)) => "unauthorized",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unavailable(_))) => "discord_unavailable",
            Self::Service(AuthError::Repository(_)) => "repository_error",
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
            Self::InvalidRequest(RequestValidationError::MissingDiscordAccessToken) => {
                "discord_access_token is required".to_string()
            }
            Self::Service(error) => error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::LoginResult;
    use crate::services::LoginUser;

    struct FakeLoginService {
        result: Result<LoginResult, AuthError>,
    }

    impl LoginService for FakeLoginService {
        fn login(&self, _command: LoginCommand) -> Result<LoginResult, AuthError> {
            self.result.clone()
        }
    }

    #[test]
    fn handle_login_returns_login_response() {
        let service = FakeLoginService {
            result: Ok(LoginResult {
                access_token: "token-1".to_string(),
                user: LoginUser {
                    discord_user_id: "1001".to_string(),
                    display_name: "asagi".to_string(),
                    avatar_hash: Some("hash".to_string()),
                    avatar_url: Some("https://cdn.discordapp.com/avatar.png".to_string()),
                },
            }),
        };

        let response = handle_login(
            &service,
            LoginRequest {
                discord_access_token: "valid-token".to_string(),
            },
        )
        .expect("handler should succeed");

        assert_eq!(response.access_token, "token-1");
        assert_eq!(response.user.display_name, "asagi");
    }

    #[test]
    fn handle_login_rejects_empty_token() {
        let service = FakeLoginService {
            result: unreachable_result(),
        };

        let error = handle_login(
            &service,
            LoginRequest {
                discord_access_token: "  ".to_string(),
            },
        )
        .expect_err("handler should fail");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn login_handler_error_builds_api_error_response() {
        let error = LoginHandlerError::InvalidRequest(RequestValidationError::MissingDiscordAccessToken);
        let response = error.to_api_error_response();

        assert_eq!(response.code, "invalid_request");
        assert_eq!(response.message, "discord_access_token is required");
    }

    fn unreachable_result() -> Result<LoginResult, AuthError> {
        Err(AuthError::InvalidRequest("not used".to_string()))
    }
}
