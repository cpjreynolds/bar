mod errors;
mod config;

pub use self::errors::{
    Error,
    Result,
};

pub use self::config::{
    Config,
    Geometry,
    Color,
};
