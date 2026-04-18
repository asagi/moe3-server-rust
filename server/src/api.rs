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
pub(crate) use handlers::handle_auth_login;

// requests
#[allow(unused_imports)]
pub(crate) use requests::AuthLoginRequest;

// responses
#[allow(unused_imports)]
pub(crate) use responses::ApiErrorResponse;
#[allow(unused_imports)]
pub(crate) use responses::AuthLoginResponse;
