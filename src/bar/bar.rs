use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::Bound;
use std::collections::btree_map::Range;
use std::process::{
    Command,
    Stdio,
    ChildStdin,
};

use pipe::PipeWriter;
use util::{
    Result,
    Error,
};

pub struct Bar {
    stdin: ChildStdin,
    elts: BTreeMap<Position, Element>,
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
            stdin: stdin,
            elts: BTreeMap::new(),
        })
    }

    pub fn stdin(&mut self) -> &mut ChildStdin {
        &mut self.stdin
    }

    pub fn insert_elt(&mut self, pos: Position, elt: Element) {
        self.elts.insert(pos, elt);
    }

    pub fn remove_elt(&mut self, pos: Position) {
        self.elts.remove(&pos);
    }

    pub fn iter_left(&mut self) -> Range<Position, Element> {
        let ref end = Position {
            align: Align::Center,
            slot: 0,
        };
        self.elts.range(Bound::Unbounded, Bound::Excluded(end))
    }
}

pub trait ToElement {
    fn to_elt(&self) -> Element;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Element(pub usize);

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

    pub fn slot(&self) -> usize {
        self.slot
    }

    pub fn align(&self) -> Align {
        self.align
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

#[derive(Debug)]
pub enum Color {
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

