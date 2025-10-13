use super::Tape;
use log::error;
use std::io;

pub struct Vm<'src> {
    ptr: usize,
    src: &'src str,
    data: Tape<u8>,
}

#[derive(Debug)]
pub enum Context {
    Loop(usize),
}

impl<'src> Vm<'src> {
    pub fn new(src: &'src str) -> Self {
        Vm {
            ptr: 0,
            src,
            data: Tape::default(),
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

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut stack: Vec<Context> = Vec::new();

        while let Some(c) = self.next_char() {
            match c {
                '0'..='9' => {
                    self.data.write(c.to_digit(10).unwrap() as u8);
                }
                '>' => self.data.right(),
                '<' => self.data.left(),
                'c' => {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    self.data.write(buf.trim().parse::<u8>()?);
                }
                'i' => {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    self.data.write(buf.trim().parse::<char>()? as u8);
                }
                'n' => {
                    print!("{}", self.data.read());
                }
                'o' => {
                    print!("{}", self.data.read() as char);
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
                    self.data.write(left * right);
                }
                '/' => {
                    let left = self.data.read();
                    self.data.right();
                    let right = self.data.read();
                    self.data.left();
                    self.data.write(left / right);
                }
                '[' => {}
                ']' => match stack.pop() {
                    None => {}
                    Some(c) => match c {
                        Context::Loop(ptr) => {
                            if self.data.read() != 0 {
                                self.seek_char(ptr);
                                stack.push(c);
                            }
                        }
                    },
                },
                'z' => {
                    if self.current_char() != Some('[') {
                        error!("'z' should have a ']' after! Ignoring.");
                    } else {
                        self.next_char();
                    }

                    if self.data.read() != 0 {
                        stack.push(Context::Loop(self.ptr));
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
        }

        Ok(())
    }
}
