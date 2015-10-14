use std::path::{
    PathBuf,
    Path,
};
use std::env;
use std::fs::File;
use std::io::prelude::*;

use toml;

use util::{
    Result,
    Error,
};

#[derive(Debug, Clone, RustcDecodable)]
pub struct Config {
    pub fonts: Vec<String>,
    pub geom: Geometry,
    pub color: Color,
}

impl Config {
    pub fn new() -> Result<Config> {
        let path = get_conf_path();
        Config::from_path(path)
    }

    pub fn from_path<P>(path: P) -> Result<Config>
        where P: AsRef<Path>
    {
        let mut cfile = try!(File::open(path));
        let mut buf = String::new();
        try!(cfile.read_to_string(&mut buf));

        toml::decode_str(&buf).ok_or(Error::new("config parse error"))
    }
}

fn get_conf_path() -> PathBuf {
    if let Ok(p) = env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(p);
        path.push("bar/bar.toml");
        path
    } else {
        let mut path = env::home_dir().unwrap();
        path.push(".config/bar/bar.toml");
        path
    }
}

#[derive(Debug, Clone, RustcDecodable)]
pub struct Geometry {
    pub size: [u32; 2],
    pub offset: [u32; 2],
}

#[derive(Debug, Clone, RustcDecodable)]
pub struct Color {
    pub fg: String,
    pub bg: String,
}

