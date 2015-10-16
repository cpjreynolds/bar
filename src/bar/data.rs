use std::process::{
    Command,
    Stdio,
};
use std::collections::HashMap;

use bar::Color;
use bar::Element;
use pipe::PipeWriter;
use util::{
    Result,
    Error,
};

pub trait Provider {
    fn is_data(&self, data: &str) -> bool;
    fn consume(&mut self, data: &str);
}

#[derive(Debug)]
pub struct WindowManager {
    dtops: Vec<Desktop>,
}

impl WindowManager {
    pub fn new(output: &PipeWriter) -> Result<WindowManager> {
        let outpipe = try!(output.stdio());
        try!(Command::new("bspc")
             .arg("control")
             .arg("--subscribe")
             .stdin(Stdio::null())
             .stdout(outpipe)
             .stderr(Stdio::inherit())
             .spawn()
             .map_err(|err| Error::from(err)));

        Ok(WindowManager {
            dtops: Vec::new(),
        })
    }
}

impl Element for WindowManager {
    fn fmt(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for d in &self.dtops {
            buf.extend(d.fmt().into_iter());
        }
        buf
    }
}

impl Provider for WindowManager {
    fn is_data(&self, data: &str) -> bool {
        data.starts_with("WM")
    }

    fn consume(&mut self, data: &str) {
        // If it is valid data, these cannot fail.
        let start = data.find(':').unwrap() + 1;
        let end = data.rfind(':').unwrap();

        let data = &data[start..end];

        for (i, dtop) in data.split(':').enumerate() {
            let mut chars = dtop.chars();
            let status = chars.next().unwrap();
            let (occupied, focused) = parse_status(status);

            if self.dtops.len() > i {
                self.dtops[i] = Desktop::new(occupied, focused);
            } else {
                self.dtops.push(Desktop::new(occupied, focused));
            }
        }

    }
}

fn parse_status(status: char) -> (bool, bool) {
    match status {
        'o' => (true, false),
        'O' => (true, true),
        'f' => (false, false),
        'F' => (false, true),
        _ => (false, false),
    }
}

#[derive(Debug, Clone)]
pub struct Desktop {
    occupied: bool,
    focused: bool,
}

impl Desktop {
    pub fn new(occupied: bool, focused: bool) -> Desktop {
        Desktop {
            occupied: occupied,
            focused: focused,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn is_occupied(&self) -> bool {
        self.occupied
    }
}

impl Element for Desktop {
    fn fmt(&self) -> Vec<u8> {
        if self.focused && self.occupied {
            format!("%{{F#{:x}}} \u{f111} %{{F-}}", Color::Violet as u32).into_bytes()
        } else if self.focused {
            format!("%{{F#{:x}}} \u{f10c} %{{F-}}", Color::Violet as u32).into_bytes()
        } else if self.occupied {
            format!(" \u{f111} ").into_bytes()
        } else {
            format!(" \u{f10c} ").into_bytes()
        }
    }
}

#[derive(Debug)]
pub struct System {
    pub stats: HashMap<String, String>,
    pub bat: Battery,
    pub datetime: DateTime,
}

#[derive(Debug)]
pub struct DateTime {
    date: String,
    time: String,
}

impl Element for DateTime {
    fn fmt(&self) -> Vec<u8> {
        format!("\u{f073} {}  \u{f017} {} ", self.date, self.time).into_bytes()
    }
}

impl Default for DateTime {
    fn default() -> DateTime {
        DateTime {
            date: String::from("Mon Jan 1"),
            time: String::from("00:00"),
        }
    }
}

#[derive(Debug)]
pub struct Battery {
    pct: String,
    pub time: String,
    pub status: BatStatus,
}

impl Battery {
    const ICON: &'static [&'static str] = &[
        "\u{f244}",
        "\u{f243}",
        "\u{f242}",
        "\u{f241}",
        "\u{f240}",
    ];

    fn pct_val(&self) -> usize {
        self.pct.parse().unwrap()
    }

    fn icon(&self) -> &'static str {
        let pct = self.pct_val();
        if pct > 75 {
            Battery::ICON[4]
        } else if pct > 50 {
            Battery::ICON[3]
        } else if pct > 25 {
            Battery::ICON[2]
        } else if pct > 10 {
            Battery::ICON[1]
        } else {
            Battery::ICON[0]
        }
    }
}

impl Element for Battery {
    fn fmt(&self) -> Vec<u8> {
        format!("%{{T4}}  {}%{{T-}} {}%%", self.icon(), self.pct).into_bytes()
    }
}

impl Default for Battery {
    fn default() -> Battery {
        Battery {
            pct: String::from("0"),
            time: String::from("0:00"),
            status: BatStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatStatus {
    Charging,
    Discharging,
    Full,
    Empty,
    Unknown,
}

impl From<char> for BatStatus {
    fn from(c: char) -> BatStatus {
        match c {
            'C' => BatStatus::Charging,
            'D' => BatStatus::Discharging,
            'F' => BatStatus::Full,
            'E' => BatStatus::Empty,
            _ => BatStatus::Unknown,
        }
    }
}

impl System {
    pub fn new(output: &PipeWriter) -> Result<System> {
        let outpipe = try!(output.stdio());

        try!(Command::new("conky")
            .stdin(Stdio::null())
            .stdout(outpipe)
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| Error::from(err)));

        Ok(System {
            stats: HashMap::new(),
            bat: Battery::default(),
            datetime: DateTime::default(),
        })
    }
}

impl Provider for System {
    fn is_data(&self, data: &str) -> bool {
        !data.starts_with("WM")
    }

    fn consume(&mut self, data: &str) {
        let mid = data.find('=').unwrap();
        let key = data[..mid].trim();
        let val = data[mid+1..].trim();

        let key = String::from(key);

        match &*key {
            "BAT_TIME" => {
                let time = parse_bat_time(val);
                self.bat.time = time;
            },
            "BAT_STATUS" => {
                let stat_char = val.chars().next().unwrap();
                self.bat.status = BatStatus::from(stat_char);

                let start = val.find(char::is_numeric).unwrap();
                let end = val.rfind('%').unwrap();
                self.bat.pct = String::from(val[start..end].trim());
            },
            "TIME" => {
                self.datetime.time = String::from(val);
            },
            "DATE" => {
                self.datetime.date = String::from(val);
            },
            _ => {
                let val = String::from(val);
                self.stats.insert(key, val);
            },
        }
    }
}

fn parse_bat_time(data: &str) -> String {
    let mut time = String::new();

    for elt in data.split_whitespace() {
        if let Some(hpos) = elt.rfind('h') {
            time.push_str(&elt[..hpos]);
        } else if let Some(mpos) = elt.rfind('m') {
            time.push(':');
            let mstr = &elt[..mpos].trim();
            if mstr.len() < 2 {
                time.push('0');
            }
            time.push_str(mstr);
        }
    }

    time
}


