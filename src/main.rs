use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs,
    path::PathBuf,
};

mod vm;

use clap::Parser;
use log::*;

use crate::vm::Vm;

#[derive(Parser)]
struct Args {
    file: PathBuf,

    #[clap(short, long)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    clang_log::init(Level::Trace, "snl");

    let src = fs::read_to_string(args.file)?;

    let mut vm = Vm::new(&src, args.debug);
    vm.run()?;

    Ok(())
}

#[derive(Default)]
pub struct Tape<T>
where
    T: Copy + Default,
{
    data: HashMap<usize, T>,
    head: usize,
}

impl<T: Copy + Default> Tape<T> {
    pub fn right(&mut self) {
        self.head += 1;
    }

    pub fn left(&mut self) {
        self.head -= 1;
    }

    pub fn read(&self) -> T {
        self.data.get(&self.head).copied().unwrap_or_default()
    }

    pub fn write(&mut self, value: T) {
        self.data.insert(self.head, value);
    }

    pub fn new() -> Self {
        Tape {
            data: HashMap::new(),
            head: 0,
        }
    }
}

impl Display for Tape<u8> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut result = vec![];
        for i in &self.data {
            while result.len() <= *i.0 * 3 + 3 {
                result.push(' ');
            }
            if !(*i.1).is_ascii_control() {
                result[*i.0 * 3] = *i.1 as char;
                result[*i.0 * 3 + 2] = '|';
            } else {
                let formatted = format!("{:X}", *i.1);
                let mut chars = formatted.chars();
                result[*i.0 * 3 + 1] = chars.next().unwrap();
                result[*i.0 * 3] = chars.next().unwrap_or('0');
                result[*i.0 * 3 + 2] = '|';
            }
        }

        f.write_str(result.into_iter().collect::<String>().as_str())?;
        f.write_str("\n")?;
        f.write_str(("   ".repeat(self.head) + "^").as_str())
    }
}

pub fn display_stack(stack: &[u8]) -> String {
    let mut result = String::with_capacity(stack.len() * 3);

    for i in stack {
        if !(*i).is_ascii_control() {
            result.push(*i as char);
            result.push_str(" |");
        } else {
            let formatted = format!("{:X}", *i);
            if formatted.len() == 1 {
                result.push('0');
            }
            result.push_str(&formatted);
            result.push('|');
        }
    }

    result
}
