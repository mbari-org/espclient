# ESP Client in Rust

What: A basic ESP client implementation in Rust.

Why: Learn and practice some basic Rust socket programming.

How: Using standard Rust libraries for socket connection and some others.
Use of more sophisticated libraries (tokio, async-std, etc.) are interesting
possible variations but not the focus of this exercise, at least initially.
Feel free to contribute if you are so inclined!

## Installing

With `cargo` on your system, you can install `espclient` as follows:

```
$ cargo install --git https://github.com/mbari-org/espclient.git
```

Of course, you can also clone this repo and then run tests, run the program, build, etc:

```
$ cargo test
$ cargo run -- --help
$ cargo build --release
```


## Running

Have an ESP server running somewhere and then launch the program indicating
the corresponding `host:port`.

Here's a session with an ESP server running on bufflehead:

```
$ espclient bufflehead.shore.mbari.org:7777
Connected to bufflehead.shore.mbari.org:7777
Using name: espclient.rs

   line: espclient.rs
 stream: Prompt
-> Cmd.status
   line: {loadedCartridge: 43,
   line:  state: :READY,
   line:  type: :archiveHiBiomass_bac}
 stream: Prompt
-> Cmd.startFiltering
   line: :FILTERING
 stream: Prompt
-> Cmd.status
   line: {loadedCartridge: 43,
   line:  state: :PRIMING,
   line:  type: :archiveHiBiomass_bac,
   line:  volumeFiltered: 0.0}
 stream: Prompt
-> Cmd.status
   line: {loadedCartridge: 43,
   line:  state: :PAUSED,
   line:  type: :archiveHiBiomass_bac,
   line:  volumeFiltered: 10.0}
 stream: Prompt
-> Cmd.startProcessing
   line: :PROCESSING
 stream: Prompt
-> Cmd.status
   line: {loadedCartridge: 43,
   line:  state: :PROCESSING,
   line:  type: :archiveHiBiomass_bac,
   line:  volumeFiltered: 10.0}
 stream: Prompt
-> Cmd.status
   line: {loadedCartridge: 43,
   line:  state: :PROCESSED,
   line:  type: :archiveHiBiomass_bac,
   line:  volumeFiltered: 10.0}
 stream: Prompt
-> Cmd.stop
   line: :STOPPED
 stream: Prompt
-> slots
   line: {42..1 => { Type: :archiveHiBiomass_bac,
   line: 	 State: :dry},
   line:  [60..57, 55..50, 48] => { Type: :archiveHiBiomass_bac,
   line: 	 State: :filtering,
   line: 	 filtered: 10},
   line:  [46, 45] => { Type: :archiveHiBiomass_bac,
   line: 	 State: :filtering,
   line: 	 filtered: 10.0},
   line:  [56, 49] => { Type: :archiveHiBiomass_bac,
   line: 	 State: :processed,
   line: 	 filtered: 10},
   line:  spare: 10,
   line:  [47, 44, 43] => { Type: :archiveHiBiomass_bac,
   line: 	 State: :processed,
   line: 	 filtered: 10.0}}
 stream: Prompt
->
Ctrl-D
```

Run `espclient --help` to get a usage message.


## Change log

- 2020-08-18: initial functional version

## Some refs

- https://github.com/jtenner/telnet_codec 
- https://stjepang.github.io/2020/04/03/why-im-building-a-new-async-runtime.html
- https://blog.burntsushi.net/rust-error-handling/#the-error-trait
