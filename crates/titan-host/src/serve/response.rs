use titan_common::ControlResponse;

pub(crate) fn server_err(code: u16, message: impl Into<String>) -> ControlResponse {
    ControlResponse::ServerError {
        code,
        message: message.into(),
    }
}
