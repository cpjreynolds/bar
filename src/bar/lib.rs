#![feature(box_syntax, box_patterns)]
#![feature(associated_consts)]
#![feature(collections, collections_bound, btree_range)]
#![feature(unicode)]

extern crate libc;
extern crate docopt;
extern crate toml;
extern crate term;
extern crate rustc_serialize;
extern crate collections;

mod bar;
pub use self::bar::Bar;
pub use self::bar::{
    Format,
    Formatter,
    Position,
    Align,
};
pub mod data;
pub mod util;
pub mod pipe;

use term::color;
use term::Terminal;
use rustc_serialize::{
    Decodable,
    Encodable,
};
use docopt::Docopt;

use util::{
    Error,
    Result,
};

pub fn execute<A, V>(exec: fn(A) -> Result<V>, usage: &str)
    where A: Decodable,
          V: Encodable,
{
    let args: A = decode_args(usage);
    match (exec)(args) {
        Ok(..) => {},
        Err(e) => handle_error(e),
    }
}

fn decode_args<A>(usage: &str) -> A
    where A: Decodable
{
    let docopt = Docopt::new(usage).unwrap()
        .help(true)
        .version(Some(::version()));

    docopt.decode().unwrap_or_else(|e| e.exit())
}

fn handle_error(err: Error) -> ! {
    if let Some(mut t) = term::stderr() {
        let _ = t.fg(color::RED);
        let _ = writeln!(t, "{}", err);
        let _ = t.reset();
    }
    ::std::process::exit(1)
}

fn version() -> String {
    format!("bar {}", env!("CARGO_PKG_VERSION"))
}
