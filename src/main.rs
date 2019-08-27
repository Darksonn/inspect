use clap::{Arg, App, ArgMatches};
use std::io::{Read, BufRead, BufReader};
use std::fs::File;

pub mod base64;

fn main() {
    std::process::exit(run());
}

static KINDS: &[&str] = &[
    "json",
    "f32",
    "f64",
    "u16",
    "u32",
    "u64",
    "i16",
    "i32",
    "i64",
];

fn build_input<'a, I: BufRead + 'a>(file: I, matches: &ArgMatches) -> Box<dyn Read + 'a> {
    if matches.is_present("base64") {
        return Box::new(base64::Base64::new(file));
    }
    Box::new(file)
}

fn run() -> i32 {
    let matches = App::new("inspect")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Alice Ryhl <alice@ryhl.io>")
        .about("Inspect various data")
        .arg(Arg::with_name("KIND")
             .help("The kind of data to read.")
             .required(true)
             .possible_values(KINDS)
             .index(1))
        .arg(Arg::with_name("FILE")
             .help("Where to read data from. Defaults to standard input.")
             .index(2))
        .arg(Arg::with_name("base64")
             .short("b")
             .long("base64")
             .help("Decode the input as base64 before passing it on."))
        .arg(Arg::with_name("hex")
             .short("h")
             .long("hex")
             .help("Decode the input as hex before passing it on."))
        .get_matches();

    let stdin;
    let mut input: Box<dyn Read> = match matches.value_of("FILE") {
        None | Some("-") => {
            stdin = std::io::stdin();
            build_input(stdin.lock(), &matches)
        },
        Some(path) => {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Unable to open {}\n{}", path, err);
                    return 4;
                },
            };
            build_input(BufReader::new(file), &matches)
        },
    };

    let mut buf = [0; 1024];
    let mut len = input.read(&mut buf).expect("Failed to read input.");
    while len > 0 {
        println!("{}", std::str::from_utf8(&buf[0..len]).unwrap());
        len = input.read(&mut buf).expect("Failed to read input.");
    }

    0
}
