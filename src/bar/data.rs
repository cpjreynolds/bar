use std::process::{
    Command,
    Stdio,
};

use bar::{
    Color,
    Format,
    Formatter,
    Icon,
};
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

impl Format for WindowManager {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        for d in &self.dtops {
            try!(fmt.write(d))
        }
        Ok(())
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

impl Format for Desktop {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        if self.focused && self.occupied {
            fmt.write(&*format!("%{{F#{:x}}} \u{f3a7} %{{F-}}", Color::Violet as u32))
        } else if self.focused {
            fmt.write(&*format!("%{{F#{:x}}} \u{f3a6} %{{F-}}", Color::Violet as u32))
        } else if self.occupied {
            fmt.write(&*format!(" \u{f3a7} "))
        } else {
            fmt.write(&*format!(" \u{f3a6} "))
        }
    }
}

#[derive(Debug, Clone)]
pub struct System {
    pub bat: Battery,
    pub datetime: DateTime,
    pub cpu: Cpu,
}

#[derive(Debug, Clone)]
pub struct DateTime {
    date: String,
    time: String,
}

impl Format for DateTime {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        fmt.write(&*format!("{} {} ", self.date, self.time))
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

#[derive(Debug, Clone)]
pub struct Battery {
    pct: usize,
    pub time: String,
    pub status: BatStatus,
}

impl Format for Battery {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        fmt.write(&*format!(" bat: {:03}", self.pct))
    }
}

impl Default for Battery {
    fn default() -> Battery {
        Battery {
            pct: 0,
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

#[derive(Debug, Clone)]
pub struct Cpu {
    temp: usize,
    freq: [f32; 4],
    usage: [usize; 4],
}

impl Format for Cpu {
    fn fmt(&self, fmt: &mut Formatter) -> Result<()> {
        fmt.write(&*format!(" temp: {:03} freq: {:.2}/{:.2}/{:.2}/{:.2} use: {:03}/{:03}/{:03}/{:03}",
                self.temp,
                self.freq[0], self.freq[1], self.freq[2], self.freq[3],
                self.usage[0], self.usage[1], self.usage[2], self.usage[3],
                ))
    }
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            temp: 0,
            freq: [0.0; 4],
            usage: [0; 4],
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
            bat: Battery::default(),
            datetime: DateTime::default(),
            cpu: Cpu::default(),
        })
    }
}

impl Provider for System {
    fn is_data(&self, data: &str) -> bool {
        // Any data that doesn't belong to the window manager.
        !data.starts_with("WM")
    }

    fn consume(&mut self, data: &str) {
        let mid = data.find('=').unwrap();
        let key = data[..mid].trim();
        let val = data[mid+1..].trim();


        match key {
            "BAT_TIME" => {
                let time = parse_bat_time(val);
                self.bat.time = time;
            },
            "BAT_STATUS" => {
                let stat_char = val.chars().next().unwrap();
                self.bat.status = BatStatus::from(stat_char);

                let start = val.find(char::is_numeric).unwrap();
                let end = val.rfind('%').unwrap();
                self.bat.pct = String::from(val[start..end].trim()).parse().unwrap();
            },
            "TIME" => {
                self.datetime.time = String::from(val);
            },
            "DATE" => {
                self.datetime.date = String::from(val);
            },
            "TEMP" => {
                self.cpu.temp = val.parse().unwrap();
            },
            "CPU" => {
                for (i, core) in val.split_whitespace().enumerate().take(4) {
                    self.cpu.usage[i] = core.parse().unwrap();
                }
            },
            "CPU_FREQ" => {
                for (i, core) in val.split_whitespace().enumerate().take(4) {
                    self.cpu.freq[i] = core.parse().unwrap();
                }
            },
            _ => {},
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
