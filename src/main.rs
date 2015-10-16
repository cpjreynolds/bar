extern crate bar;
extern crate toml;
extern crate libc;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::io::{
    BufReader,
};

use bar::pipe;
use bar::{
    Bar,
    Position,
};
use bar::util::Result;
use bar::util::Config;
use bar::data::{
    Provider,
    System,
    WindowManager,
};

static USAGE: &'static str = "
bar

Usage:
    bar [options]

Options:
    -c, --config=<path>     Specify config file path.
    -h, --help              Display this message.
    -v, --verbose           Print version info and exit.

Configuration:
    Default config path is `$XDG_CONFIG_HOME/bar/bar.toml`.

";

#[derive(Debug, Clone, RustcDecodable)]
pub struct Args {
    flag_config: Option<String>,
}

fn main() {
    bar::execute(execute, USAGE);
}

fn execute(args: Args) -> Result<()> {
    let conf = if let Some(path) = args.flag_config {
        try!(Config::from_path(path))
    } else {
        try!(Config::new())
    };
    let args = gen_args(&conf);

    let (rd_pipe, wr_pipe) = try!(pipe::pipe());

    let mut bar = try!(Bar::new(&wr_pipe, &args));

    let mut sys = try!(System::new(&wr_pipe));
    let mut wm = try!(WindowManager::new(&wr_pipe));
    let input = BufReader::new(rd_pipe);

    for line in input.lines() {
        let line = try!(line);

        if wm.is_data(&line) {
            wm.consume(&line);
        } else if sys.is_data(&line) {
            sys.consume(&line);
        }

        // Date.
        bar.register(Position::right(), &sys.datetime);

        // Battery.
        bar.register(Position::left(), &sys.bat);

        bar.register(Position::center(), &wm);

        try!(bar.flush());
    }
    Ok(())
}

fn gen_args(conf: &Config) -> Vec<String> {
    let mut args = Vec::new();

    // Size
    args.push(String::from("-g"));
    let mut size_arg = String::new();
    size_arg.push_str(&conf.geom.size[0].to_string());
    size_arg.push('x');
    size_arg.push_str(&conf.geom.size[1].to_string());
    size_arg.push('+');
    size_arg.push_str(&conf.geom.offset[0].to_string());
    size_arg.push('+');
    size_arg.push_str(&conf.geom.offset[1].to_string());
    args.push(size_arg);

    // Fonts
    for font in &conf.fonts {
        args.push(String::from("-f"));
        args.push(font.clone());
    }

    // Colors
    args.push(String::from("-B"));
    args.push(conf.color.bg.clone());
    args.push(String::from("-F"));
    args.push(conf.color.fg.clone());

    // Underline
    args.push(String::from("-u"));
    args.push(String::from("5"));

    args
}

