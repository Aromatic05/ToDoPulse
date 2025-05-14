use serde::Serialize;
use ts_rs::TS;
use log::error;
use anyhow::Error;

#[derive(Debug,TS, Serialize)]
#[ts(export)]
#[allow(dead_code)]
pub enum ErrorKind {
    InitError,
    NotFound,
    AlreadyExists,
    InvalidInput,
    PermissionDenied,
    DatabaseError,
    IoError,
    InternetError,
    UnknownError,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InitError => write!(f, "Initialization error"),
            ErrorKind::NotFound => write!(f, "Not found"),
            ErrorKind::AlreadyExists => write!(f, "Already exists"),
            ErrorKind::InvalidInput => write!(f, "Invalid input"),
            ErrorKind::PermissionDenied => write!(f, "Permission denied"),
            ErrorKind::DatabaseError => write!(f, "Database error"),
            ErrorKind::IoError => write!(f, "IO error"),
            ErrorKind::InternetError => write!(f, "Internet error"),
            ErrorKind::UnknownError => write!(f, "Unknown error"),
        }
    }
    
}

impl std::error::Error for ErrorKind{}

impl From<anyhow::Error> for ErrorKind {
    fn from(e: anyhow::Error) -> Self {
        error!("Anyhow error: {:?}", e);
        if let Some(io_err) = Error::downcast_ref::<std::io::Error>(&e) {
            match io_err.kind() {
                std::io::ErrorKind::NotFound => ErrorKind::NotFound,
                std::io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
                _ => ErrorKind::IoError,
            }
        } else if let Some(_db_err) = Error::downcast_ref::<redb::Error>(&e) {
            ErrorKind::DatabaseError
        } else  {
            ErrorKind::UnknownError
        }
    }
    
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        error!("IO error: {:?}", e);
        match e.kind() {
            std::io::ErrorKind::NotFound => ErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
            _ => ErrorKind::IoError,
        }
    }
    
}

impl From<reqwest::Error> for ErrorKind {
    fn from(e: reqwest::Error) -> Self {
        error!("Reqwest error: {:?}", e);
        if e.is_timeout() {
            ErrorKind::InternetError
        } else {
            ErrorKind::UnknownError
        }
    }
    
}