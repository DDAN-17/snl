use super::Tape;
use crate::display_stack;
use anyhow::Context as _;
use colored::Colorize;
use crossterm::{
    cursor,
    terminal::{self, ClearType},
};
use log::error;
use std::io::{self, Write};

pub struct Vm<'src> {
    ptr: usize,
    src: &'src str,
    data: Tape<u8>,
    debug: bool,
    context_stack: Vec<Context>,
    stack: Vec<u8>,
}

#[derive(Debug)]
pub enum Context {
    Zero(usize),
    While(usize),
}

impl<'src> Vm<'src> {
    pub fn new(src: &'src str, debug: bool) -> Self {
        Vm {
            ptr: 0,
            src,
            data: Tape::default(),
            debug,
            context_stack: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn current_char(&self) -> Option<char> {
        self.src.chars().nth(self.ptr)
    }

    pub fn next_char(&mut self) -> Option<char> {
        let c = self.current_char();
        self.ptr += 1;
        c
    }

    pub fn seek_char(&mut self, i: usize) {
        self.ptr = i;
    }

    pub fn debug(&mut self, stdout: &str) -> anyhow::Result<()> {
        crossterm::execute!(
            io::stdout(),
            terminal::Clear(ClearType::Purge),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        print!("{stdout}");
        if !stdout.ends_with("\n") {
            println!("{}\n", "%".black().on_white());
        } else {
            println!();
        }

        println!("{}", self.src);
        println!("{}^", " ".repeat(self.ptr - 1));

        println!();

        println!("{}", self.data);

        println!("{}", display_stack(&self.stack));
        println!();

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut stdout = String::new();

        while let Some(c) = self.next_char() {
            if self.debug {
                self.debug(&stdout)?;
            }

            match c {
                '0'..='9' => {
                    self.data.write(c.to_digit(10).unwrap() as u8);
                }
                '>' => self.data.right(),
                '<' => self.data.left(),
                'c' => {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    self.data
                        .write(buf.trim().parse::<u8>().context("bad number input!")?);
                }
                'i' => {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    self.data
                        .write(buf.trim().parse::<char>().context("bad character input!")? as u8);
                }
                's' => {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    let trimmed = buf.trim();
                    for c in trimmed.bytes() {
                        self.data.write(c);
                        self.data.right();
                    }
                    self.data.write(0);
                    self.data.head -= trimmed.len();
                }
                'p' => {
                    let mut i = 0;
                    while self.data.read() != 0 {
                        let print = format!("{}", self.data.read() as char);
                        if self.debug {
                            stdout += print.as_str();
                        } else {
                            print!("{print}");
                        }

                        i += 1;
                        self.data.right();
                    }
                    self.data.head -= i;
                    io::stdout().flush()?;
                }
                'n' => {
                    let print = format!("{}", self.data.read());
                    if self.debug {
                        stdout += print.as_str();
                    } else {
                        print!("{print}");
                    }
                    io::stdout().flush()?;
                }
                'o' => {
                    let print = format!("{}", self.data.read() as char);
                    if self.debug {
                        stdout += print.as_str();
                    } else {
                        print!("{print}");
                    }
                    io::stdout().flush()?;
                }
                '+' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left + right);
                }
                '-' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left - right);
                }
                '*' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    if let Some(v) = left.checked_mul(right) {
                        self.data.write(v);
                    } else {
                        error!("Cannot multiply {left} * {right}!");
                    }
                }
                '/' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left / right);
                }
                '[' => {}
                ']' => match self.context_stack.pop() {
                    None => {}
                    Some(c) => match c {
                        Context::Zero(ptr) => {
                            if self.data.read() != 0 {
                                self.seek_char(ptr);
                                self.context_stack.push(c);
                            }
                        }
                        Context::While(ptr) => {
                            if self.data.read() == 0 {
                                self.seek_char(ptr);
                                self.context_stack.push(c);
                            }
                        }
                    },
                },
                '@' => {
                    self.stack.push(self.data.read());
                }
                '#' => {
                    if let Some(v) = self.stack.pop() {
                        self.data.write(v);
                    }
                }
                'e' => {
                    if self.current_char() != Some('[') {
                        error!("'e' should have a ']' after! Ignoring.");
                    } else {
                        self.next_char();
                    }

                    if self.data.read() == 0 {
                        let mut stack_size = 0;
                        while let Some(c) = self.next_char() {
                            if c == ']' && stack_size == 0 {
                                break;
                            } else if c == ']' {
                                stack_size -= 1;
                            } else if c == '[' {
                                stack_size += 1;
                            }
                        }
                    }
                }
                'f' => {
                    if self.current_char() != Some('[') {
                        error!("'f' should have a ']' after! Ignoring.");
                    } else {
                        self.next_char();
                    }

                    if self.data.read() != 0 {
                        let mut stack_size = 0;
                        while let Some(c) = self.next_char() {
                            if c == ']' && stack_size == 0 {
                                break;
                            } else if c == ']' {
                                stack_size -= 1;
                            } else if c == '[' {
                                stack_size += 1;
                            }
                        }
                    }
                }
                'w' => {
                    if self.current_char() != Some('[') {
                        error!("'w' should have a ']' after! Ignoring.");
                    } else {
                        self.next_char();
                    }

                    if self.data.read() == 0 {
                        self.context_stack.push(Context::While(self.ptr));
                    } else {
                        let mut stack_size = 0;
                        while let Some(c) = self.next_char() {
                            if c == ']' && stack_size == 0 {
                                break;
                            } else if c == ']' {
                                stack_size -= 1;
                            } else if c == '[' {
                                stack_size += 1;
                            }
                        }
                    }
                }
                'z' => {
                    if self.current_char() != Some('[') {
                        error!("'z' should have a ']' after! Ignoring.");
                    } else {
                        self.next_char();
                    }

                    if self.data.read() != 0 {
                        self.context_stack.push(Context::Zero(self.ptr));
                    } else {
                        let mut stack_size = 0;
                        while let Some(c) = self.next_char() {
                            if c == ']' && stack_size == 0 {
                                break;
                            } else if c == ']' {
                                stack_size -= 1;
                            } else if c == '[' {
                                stack_size += 1;
                            }
                        }
                    }
                }
                _ => error!("Unknown character '{c}'! Skipping."),
            }

            if self.debug {
                io::stdin().read_line(&mut String::new())?;
            }
        }
        if self.debug {
            self.debug(&stdout)?;
        }

        Ok(())
    }
}
