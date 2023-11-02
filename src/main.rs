mod decoder;
mod encoder;
mod error;
mod event;

use decoder::EspDecoder;
use encoder::encode_line;
use event::*;

use bytes::{BufMut, BytesMut};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use clap::Parser;

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const PROMPT: &str = "-> ";
const HISTORY_FILE: &str = "history.txt";

fn cli_styles() -> clap::builder::Styles {
    use anstyle::{
        AnsiColor::{self, *},
        Color, Style,
    };
    fn style(color: AnsiColor) -> Style {
        Style::new().bold().fg_color(Some(Color::Ansi(color)))
    }
    clap::builder::Styles::styled()
        .usage(style(Yellow).underline())
        .header(style(Yellow).underline())
        .literal(style(Green))
        .placeholder(style(Blue))
}

#[derive(Parser, Debug)]
#[clap(version, about = "ESP Client in Rust", long_about = None)]
#[command(styles=cli_styles())]
// #[command(styles=clap::builder::Styles::styled())]
struct Opts {
    /// host:port indicating the running ESP server to connect to
    #[arg()]
    server: String,

    /// My name as client for ESP server's log
    #[arg(short, long, default_value = "espclient.rs")]
    name: String,

    /// Command beginning interactive session
    #[arg(short, long, default_value = "showlog 0")]
    cmd: String,

    /// Simple output (by default, show stream multiplexing explicitly)
    #[arg(short, long)]
    simple: bool,

    /// Summarize raw socket traffic on STDERR
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let opts = Opts::parse();

    match TcpStream::connect(&opts.server) {
        Ok(stream) => connected(&opts, stream),

        Err(e) => eprintln!("Error connecting to {}: {:?}", &opts.server, e),
    }
}

fn connected(opts: &Opts, mut stream: TcpStream) {
    let simple = opts.simple;
    let debug = opts.debug;
    println!("Connected to {}", &opts.server);
    let mut from_server = stream.try_clone().unwrap();

    let (done_sender, done_receiver) = mpsc::channel();

    let from_server_thread = thread::spawn(move || {
        let mut buf = BytesMut::with_capacity(2048);

        let _ = from_server.set_nonblocking(true);

        let mut dec = EspDecoder::new(4096);

        loop {
            // are we done? (see stdin loop below)
            match done_receiver.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            // get data from server:
            let mut data = [0u8; 1024];
            match from_server.read(&mut data) {
                Ok(read_len) => {
                    // update buffer and decode to ESP events:
                    buf.put(&data[0..read_len]);
                    loop {
                        match dec.decode(&mut buf, debug) {
                            Ok(Some(event)) => {
                                match event {
                                    EspEvent::Line(line) => {
                                        if simple {
                                            println!("\r{}", line);
                                        } else {
                                            let s = dec.get_current_stream();
                                            println!(
                                                "\r{:>12} {}",
                                                format!("<{:?}> |", s).cyan(),
                                                line
                                            );
                                        }
                                    }
                                    EspEvent::Stream(_) => {}
                                }
                                print!("\r{}", PROMPT);
                                let _ = io::stdout().flush();
                            }

                            Ok(None) => break,

                            Err(e) => println!("decode error: {:?}", e),
                        }
                    }
                }

                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(200));
                }

                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    });

    let to_server = &mut stream;
    stdin_loop(opts, done_sender, to_server, from_server_thread).unwrap();
}

fn stdin_loop(
    opts: &Opts,
    done_sender: mpsc::Sender<()>,
    to_server: &TcpStream,
    from_server_thread: thread::JoinHandle<()>,
) -> Result<()> {
    let mut rl = DefaultEditor::new()?; // `()` can be used when no completer is required
    if rl.load_history(HISTORY_FILE).is_err() {
        println!("{}", "(no previous history)".bright_black());
    }

    send_line(&opts.name, to_server, opts.debug);
    if !opts.cmd.trim().is_empty() {
        let _ = rl.add_history_entry(&opts.cmd);
        send_line(&opts.cmd, to_server, opts.debug);
    }

    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    if line == "exit" {
                        exit("exiting...", done_sender, from_server_thread);
                        break;
                    } else {
                        let _ = rl.add_history_entry(line);
                        send_line(line, to_server, opts.debug);
                    }
                }
            }

            Err(ReadlineError::Eof) => {
                exit("Ctrl-D", done_sender, from_server_thread);
                break;
            }

            Err(ReadlineError::Interrupted) => {
                exit("Ctrl-C", done_sender, from_server_thread);
                break;
            }

            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(HISTORY_FILE)
}

fn send_line(line: &str, mut to_server: &TcpStream, debug: bool) {
    let encoded = encode_line(line);
    to_server.write_all(&encoded).unwrap();
    to_server.flush().unwrap();
    if debug {
        debug_buffer("SENT", &encoded, false);
    }
}

fn exit(msg: &str, done_sender: mpsc::Sender<()>, from_server_thread: thread::JoinHandle<()>) {
    println!("{}", msg.bright_black());
    let _ = done_sender.send(());
    let _ = from_server_thread.join();
}

pub fn debug_buffer(prefix: &str, buffer: &[u8], add_new_line: bool) {
    eprintln!(
        "[{}: {}{}]",
        prefix,
        escape(buffer),
        if add_new_line { "\\n" } else { "" }
    );
}

fn escape(v: &[u8]) -> String {
    v.iter()
        .map(|b| match b {
            _ if 32u8 <= *b && *b <= 126u8 => format!("{}", *b as char),
            b'\t' => "\\t".to_string(),
            b'\n' => "\\n".to_string(),
            b'\r' => "\\r".to_string(),
            _ => format!("\\{:03o}", b),
        })
        .collect()
}
