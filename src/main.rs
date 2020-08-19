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
use rustyline::Editor;

use structopt::clap::crate_version;
use structopt::StructOpt;

use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const PROMPT: &str = "-> ";
const HISTORY_FILE: &str = "history.txt";

#[derive(StructOpt, Debug)]
#[structopt(name = "espclient", about = "ESP Client in Rust")]
#[structopt(version = crate_version!())]
struct Opts {
    /// host:port indicating the running ESP server
    #[structopt()]
    server: String,

    /// My name as client for ESP server's log
    #[structopt(short, long, default_value = "espclient.rs")]
    name: String,

    /// Command beginning interactive session
    #[structopt(short, long, default_value = "showlog 0")]
    cmd: String,

    /// Simple output (by default, show stream multiplexing explicitly)
    #[structopt(short, long)]
    simple: bool,
}

fn main() {
    let opts = Opts::from_args();

    match TcpStream::connect(&opts.server) {
        Ok(stream) => connected(&opts, stream),

        Err(e) => eprintln!("Error connecting to {}: {:?}", &opts.server, e),
    }
}

fn connected(opts: &Opts, mut stream: TcpStream) {
    let simple = opts.simple;
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
                        match dec.decode(&mut buf) {
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
    stdin_loop(&opts, done_sender, to_server, from_server_thread);
}

fn stdin_loop(
    opts: &Opts,
    done_sender: mpsc::Sender<()>,
    to_server: &TcpStream,
    from_server_thread: thread::JoinHandle<()>,
) {
    let mut rl = Editor::<()>::new(); // `()` can be used when no completer is required
    if rl.load_history(HISTORY_FILE).is_err() {
        println!("{}", "(no previous history)".bright_black());
    }

    send_line(&opts.name, to_server);
    if opts.cmd.trim().len() > 0 {
        rl.add_history_entry(&opts.cmd);
        send_line(&opts.cmd, to_server);
    }

    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.len() > 0 {
                    if line == "exit" {
                        exit("exiting...", done_sender, from_server_thread);
                        break;
                    } else {
                        rl.add_history_entry(line);
                        send_line(&line, to_server);
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

    rl.save_history(HISTORY_FILE).unwrap();
}

fn send_line(line: &str, mut to_server: &TcpStream) {
    to_server.write(&encode_line(&line)).unwrap();
    to_server.flush().unwrap();
}

fn exit(msg: &str, done_sender: mpsc::Sender<()>, from_server_thread: thread::JoinHandle<()>) {
    println!("{}", msg.bright_black());
    let _ = done_sender.send(());
    let _ = from_server_thread.join();
}
