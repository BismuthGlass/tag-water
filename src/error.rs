use rocket::response::{self, Response, Responder};
use rocket::Request;
use rocket::http::Status;

pub struct SqliteError(sqlite::Error);

impl<'r> Responder<'r, 'static> for SqliteError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let code = self.0.code.or(Some(1)).unwrap();
        let message = self.0.message.or(Some("Unknown SQLITE error".to_string())).unwrap();
        Response::build_from(format!("{}: {}", code, message).respond_to(req)?)
            .status(Status::InternalServerError)
            .ok()
    }
}

pub struct GeneralError {
    pub code: i32,
    pub message: String,
}

impl<'r> Responder<'r, 'static> for GeneralError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(format!("{}: {}", self.code, self.message).respond_to(req)?)
            .status(Status::InternalServerError)
            .ok()
    }
}

#[derive(rocket::response::Responder)]
pub enum Error {
    General(GeneralError),
    Sqlite(SqliteError),
    Io(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(code: i32, message: &str) -> Error {
        Error::General(GeneralError {
            code,
            message: message.to_string(),
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Error {
        Error::Io(value)
    }
}

impl From<sqlite::Error> for Error {
    fn from(value: sqlite::Error) -> Error {
        Error::Sqlite(SqliteError(value))
    }
}


