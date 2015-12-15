use std::char;
use std::str;
use std::fmt;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::Bound;
use std::process::{
    Command,
    Stdio,
    ChildStdin,
};
use std::io::prelude::*;
use std::io::{
    BufWriter,
};

use pipe::PipeWriter;
use util::{
    Result,
    Error,
};

pub struct Bar {
    stdin: BufWriter<ChildStdin>,
    elts: BTreeMap<Position, Vec<u8>>,
}

impl Bar {
    pub fn new(output: &PipeWriter, args: &[String]) -> Result<Bar> {
        let outpipe = try!(output.stdio());
        let bar = try!(Command::new("lemonbar")
                       .args(args)
                       .stdin(Stdio::piped())
                       .stdout(outpipe)
                       .stderr(Stdio::inherit())
                       .spawn()
                       .map_err(|err| Error::from(err)));

        let stdin = try!(bar.stdin.ok_or(Error::new("failed to grab `lemonbar` stdin")));

        Ok(Bar {
            stdin: BufWriter::new(stdin),
            elts: BTreeMap::new(),
        })
    }

    pub fn register<T>(&mut self, pos: Position, elt: &T)
        where T: Format
    {
        let mut buf = Vec::new();
        {
            let mut fmtr = Formatter { buf: &mut buf };
            elt.fmt(&mut fmtr);
        }
        self.elts.insert(pos, buf);
    }

    pub fn deregister(&mut self, pos: Position) {
        self.elts.remove(&pos);
    }

    pub fn flush(&mut self) -> Result<()> {
        // Left.
        try!(self.stdin.write_all(b"%{l}"));
        for (_, elt) in self.elts.range(Bound::Unbounded, Bound::Excluded(&Position::center())) {
            try!(self.stdin.write_all(elt));
        }

        // Center.
        try!(self.stdin.write_all(b"%{c}"));
        for (_, elt) in self.elts.range(Bound::Included(&Position::center()),
                                        Bound::Excluded(&Position::right()))
        {
            try!(self.stdin.write_all(elt));
        }

        // Right.
        try!(self.stdin.write_all(b"%{r}"));
        for (_, elt) in self.elts.range(Bound::Included(&Position::right()), Bound::Unbounded) {
            try!(self.stdin.write_all(elt));
        }

        try!(self.stdin.write_all(b"\n"));
        try!(self.stdin.flush());
        Ok(())
    }
}

pub trait Format {
    fn fmt(&self, &mut Formatter) -> Result<()>;
}

impl Format for str {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        try!(fmt.buf.write(self.as_bytes()));
        Ok(())
    }
}

impl Format for char {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        let mut utf8 = [0u8; 4];
        let n = self.encode_utf8(&mut utf8).unwrap_or(0);
        fmt.write(unsafe { str::from_utf8_unchecked(&utf8[..n]) })
    }
}

pub struct Formatter<'a> {
    buf: &'a mut (Write+'a),
}

impl<'a> Formatter<'a> {
    pub fn write<T: ?Sized>(&mut self, source: &T) -> Result<()>
        where T: Format
    {
        source.fmt(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    align: Align,
    slot: usize,
}

impl Position {
    pub fn new(align: Align, slot: usize) -> Position {
        Position {
            align: align,
            slot: slot,
        }
    }

    pub fn left() -> Position {
        Position {
            align: Align::Left,
            slot: 0,
        }
    }

    pub fn center() -> Position {
        Position {
            align: Align::Center,
            slot: 0,
        }
    }

    pub fn right() -> Position {
        Position {
            align: Align::Right,
            slot: 0,
        }
    }

    // Slots are from left to right.
    pub fn slot(&self) -> usize {
        self.slot
    }

    pub fn set_slot(&mut self, slot: usize) -> &mut Position {
        self.slot = slot;
        self
    }

    pub fn align(&self) -> Align {
        self.align
    }

    pub fn set_align(&mut self, align: Align) -> &mut Position {
        self.align = align;
        self
    }

    pub fn is_left(&self) -> bool {
        self.align == Align::Left
    }

    pub fn is_center(&self) -> bool {
        self.align == Align::Center
    }

    pub fn is_right(&self) -> bool {
        self.align == Align::Right
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.align > other.align {
            Some(Ordering::Greater)
        } else if self.align == other.align {
            Some(self.slot.cmp(&other.slot))
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        // `PartialOrd` implementation ensures this cannot fail.
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Format for Align {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        let align = match *self {
            Align::Left => "%{l}",
            Align::Center => "%{c}",
            Align::Right => "%{r}",
        };
        fmt.write(align)
    }
}

pub struct Styled<T> {
    inner: T,
    fg: Color,
    bg: Color,
}

impl<T> Styled<T>
    where T: Format
{
    pub fn set_fg(&mut self, fg: Color) -> &mut Self {
        self.fg = fg;
        self
    }

    pub fn set_bg(&mut self, bg: Color) -> &mut Self {
        self.bg = bg;
        self
    }
}

impl<T> From<T> for Styled<T>
    where T: Format
{
    fn from(inner: T) -> Styled<T> {
        Styled {
            inner: inner,
            fg: Color::Default,
            bg: Color::Default,
        }
    }
}

impl<T> Format for Styled<T>
    where T: Format
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        let start = format!("%{{F{} B{}}}", self.fg, self.bg);
        let end = "%{F- B-}";

        try!(fmt.write(&*start));
        try!(fmt.write(&self.inner));
        try!(fmt.write(end));
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Default,
    Base03 = 0x002b36,
    Base02 = 0x073642,
    Base01 = 0x586e75,
    Base00 = 0x657b83,
    Base0 = 0x839496,
    Base1 = 0x93a1a1,
    Base2 = 0xeee8d5,
    Base3 = 0xfdf6e3,
    Yellow = 0xb58900,
    Orange = 0xcb4b16,
    Red = 0xdc322f,
    Magenta = 0xd33682,
    Violet = 0x6c71c4,
    Blue = 0x268bd2,
    Cyan = 0x2aa198,
    Green = 0x859900,
}

impl fmt::Display for Color {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if *self == Color::Default {
            fmt.write_str("-")
        } else {
            fmt.write_str(&format!("{:x}", *self as u32))
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Icon {
    DotOn = 0xf3a6,
    DotOff = 0xf3a7,
    BatEmpty = 0xf112,
    BatLow = 0xf115,
    BatHalf = 0xf114,
    BatFull = 0xf113,
    BatCharging = 0xf111,
}

impl Format for Icon {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        let c = try!(char::from_u32(*self as u32).ok_or(Error::new("internal char error")));
        fmt.write(&c)
    }
}

