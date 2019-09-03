#[macro_use]
extern crate clap;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rcon_protocol;
extern crate termion;

use crate::termion::input::TermRead;
use clap::{App, Arg};
use log::LevelFilter;
use std::io::{stdin, stdout, ErrorKind, Read, Write};
// use termion::event::{Event, Key, MouseEvent};
use termion::{color, style};
// use termion::screen::*;

use rcon_protocol::prelude::*;

fn main() -> std::io::Result<()> {
    let stdout = stdout();
    let stdout = stdout.lock();
    let stdin = stdin();
    let stdin = stdin.lock();
    let mut my_app = CliApp::new(stdin, stdout);
    let mut builder = pretty_env_logger::formatted_builder();
    let matches = init_app();
    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match matches.occurrences_of("v") {
        1 => builder.filter(None, LevelFilter::Debug),
        2 => builder.filter(None, LevelFilter::Trace),
        _ => builder.filter(None, LevelFilter::Error),
    };
    builder.init();
    let host = matches.value_of("host").expect("remote HOST is epxected");
    let port: u16 = matches
        .value_of("port")
        .unwrap_or("25575")
        .parse()
        .expect("Expecting port to be and Unsigned Integer");
    let mut rcon = match RconClient::new(host.to_string(), Some(port)) {
        Ok(r) => r,
        Err(_) => std::process::exit(1),
    };
    writeln!(
        my_app.stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
    )
    .unwrap();
    // writeln!(
    //     my_app.stdout,
    //     "{}{}q to exit. Type stuff, use alt, and so on.{}",
    //     termion::clear::All,
    //     termion::cursor::Goto(1, 1),
    //     termion::cursor::Hide
    // )
    // .unwrap();
    // my_app.stdout.flush().unwrap();
    // for c in my_app.stdin.keys() {
    //     write!(my_app.stdout,
    //            "{}{}",
    //            termion::cursor::Goto(1, 1),
    //            termion::clear::CurrentLine)
    //             .unwrap();

    //     match c.unwrap() {
    //         Key::Char('q') => break,
    //         Key::Char(c) => println!("{}", c),
    //         Key::Alt(c) => println!("^{}", c),
    //         Key::Ctrl(c) => println!("*{}", c),
    //         Key::Esc => println!("ESC"),
    //         Key::Left => println!("←"),
    //         Key::Right => println!("→"),
    //         Key::Up => println!("↑"),
    //         Key::Down => println!("↓"),
    //         Key::Backspace => println!("×"),
    //         _ => {}
    //     }
    //     my_app.stdout.flush().unwrap();
    // }

    // write!(my_app.stdout, "{}", termion::cursor::Show).unwrap();
    // for c in my_app.stdin.events() {
    //     let evt = c.unwrap();
    //     match evt {
    //         Event::Key(Key::Char('q')) => {
    //             debug!("Bye Bye");
    //         }
    //         Event::Key(Key::Char(c)) => {
    //             debug!("Char printer: {:?}", c);
    //         }

    //         x => {
    //             debug!("Unhandled event:{:?}",x);
    //         }
    //     }
    //     my_app.stdout.flush().unwrap();
    // }
    // Ok({})
    loop {
        let out = format!("{}password:{} ", style::Bold, style::Reset);
        my_app.log(out.as_bytes());
        let mut ask_password = || -> String {
            match my_app.stdin.read_passwd(&mut my_app.stdout) {
                Ok(input) => {
                    let password = input.unwrap();
                    password.trim_end().to_owned()
                }
                Err(error) => panic!(error),
            }
        };
        let password = match matches.value_of("password") {
            Some(p) => {
                if p.is_empty() {
                    ask_password()
                } else {
                    p.to_owned()
                }
            }
            None => ask_password(),
        };
        my_app.log(b"\n");
        let result = rcon.auth(Some(password));
        if !result {
            println!("Invalid password");
        } else {
            break;
        }
    }
    loop {
        my_app.log(b"> ");
        let command = match my_app.stdin.read_line() {
            Ok(input) => {
                let input = input.unwrap();
                if input.trim().is_empty() {
                    continue;
                }
                debug!("Entered command is: {:?}", input.trim_end());
                input.trim_end().to_owned()
            }
            Err(error) => panic!(error),
        };
        let packet = match rcon.run(command.clone()) {
            Ok(p) => p,
            Err(_) => {
                info!("Connection was dropped. Trying to reconnect");
                rcon.reconnect()?;
                rcon.run(command)?
                // if e.kind() == ErrorKind::NotConnected {
                //     std::process::exit(1);
                // }
            }
        };
        let out_string = String::from_utf8(packet.payload).unwrap();
        // println!("{}{}{}", style::Bold, out_string, style::Reset);
        my_app.log(out_string.as_ref());
        my_app.log(b"\n");
    }
}

fn init_app() -> clap::ArgMatches<'static> {
    App::new("RCON client")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Simple RCON client")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .help("Sets a host")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .short("P")
                .long("port")
                .value_name("PORT")
                .help("Set a port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches()
}

struct CliApp<R, W>
where
    R: Read,
    W: Write,
{
    pub stdin: R,
    pub stdout: W,
}

impl<R, W> CliApp<R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(stdin: R, stdout: W) -> Self {
        CliApp { stdin, stdout }
    }

    pub fn log(&mut self, msg: &[u8]) {
        self.stdout.write_all(msg).unwrap();
        self.stdout.flush().unwrap()
        // print!("{}password:{} ", style::Bold, style::Reset);
    }
    pub fn info(&mut self, msg: &'static str) {
        let out = format!(
            "{}{}{}{}{}",
            style::Bold,
            color::Fg(color::Green),
            msg,
            color::Fg(color::Reset),
            style::Reset
        );
        self.log(out.as_bytes());
    }
}
