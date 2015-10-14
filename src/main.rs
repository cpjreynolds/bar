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
    Element,
    Position,
    Align,
};
use bar::util::Result;
use bar::util::Config;
use bar::data::{
    Provider,
    WindowManager,
    Desktop,
    System,
    Battery,
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

    let e0 = Element(0);
    let e1 = Element(0);
    let e2 = Element(0);
    let e3 = Element(0);
    let e4 = Element(0);

    let p0 = Position::new(Align::Left, 0);
    let p1 = Position::new(Align::Left, 3);
    let p2 = Position::new(Align::Center, 2);
    let p3 = Position::new(Align::Right, 0);
    let p4 = Position::new(Align::Right, 1);

    bar.insert_elt(p0, e0);
    bar.insert_elt(p1, e1);
    bar.insert_elt(p2, e2);
    bar.insert_elt(p3, e3);
    bar.insert_elt(p4, e4);

    for elt in bar.iter_left() {
        println!("{:?}", elt);
    }

    let mut sys = try!(System::new(&wr_pipe));
    let mut wm = try!(WindowManager::new(&wr_pipe));
    let output = bar.stdin();
    let input = BufReader::new(rd_pipe);

    for line in input.lines() {
        let line = try!(line);

        let mut outbuf = Vec::new();

        if wm.is_data(&line) {
            wm.consume(&line);
        } else if sys.is_data(&line) {
            sys.consume(&line);
        }

        // Date.
        try!(sys.datetime.write_into(&mut outbuf));

        // Battery.
        try!(sys.bat.write_into(&mut outbuf));

        try!(wm.write_into(&mut outbuf));

        outbuf.push(b'\n');
        try!(output.write(&outbuf));
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

