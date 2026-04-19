// modules
#[cfg_attr(not(test), allow(dead_code))]
mod handlers;
#[cfg_attr(not(test), allow(dead_code))]
mod requests;
#[cfg_attr(not(test), allow(dead_code))]
mod responses;

// handlers
#[allow(unused_imports)]
pub(crate) use handlers::AuthHandlerError;
#[allow(unused_imports)]
pub(crate) use handlers::CreateGameHandlerError;
#[allow(unused_imports)]
pub(crate) use handlers::handle_auth_login;
#[allow(unused_imports)]
pub(crate) use handlers::handle_create_game;

// requests
#[allow(unused_imports)]
pub(crate) use requests::AuthLoginRequest;
#[allow(unused_imports)]
pub(crate) use requests::CreateGameRequest;

// responses
#[allow(unused_imports)]
pub(crate) use responses::ApiErrorResponse;
#[allow(unused_imports)]
pub(crate) use responses::AuthLoginResponse;
#[allow(unused_imports)]
pub(crate) use responses::AuthLoginUserResponse;
#[allow(unused_imports)]
pub(crate) use responses::CreateGameResponse;
