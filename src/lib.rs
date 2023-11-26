use clap::{Arg, Command};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::{error::Error, u64};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: u64,
    bytes: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let cmd = Command::new("headr")
        .version("0.1.0")
        .author("Kurt Fehlhauer")
        .about("Rust head")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .num_args(1..)
                .default_value("-"),
        )
        .arg(
            Arg::new("lines")
                .value_name("LINES")
                .help("Number lines [default: 10]")
                .short('n')
                .long("lines")
                .value_parser(clap::value_parser!(u64).range(1..))
                .default_value("10"),
        )
        .arg(
            Arg::new("bytes")
                .value_name("BYTES")
                .help("Number of bytes")
                .short('c')
                .long("bytes")
                .conflicts_with("lines")
                .value_parser(clap::value_parser!(u64).range(1..)),
        )
        .get_matches();

    Ok(Config {
        files: cmd
            .get_many::<String>("files")
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect(),
        lines: cmd.get_one("lines").cloned().unwrap(),
        bytes: cmd.get_one("bytes").cloned(),
    })
}

#[allow(dead_code)]
fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3 is an OK integer
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Any string is an error
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // zero is an error
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let mut is_first_time = true;
    let file_count = config.files.len();

    for filename in config.files {
        match open(&filename) {
            Err(err) => {
                is_first_time = false;
                eprintln!("head: {}: {}", filename, err);
            }
            Ok(mut file) => {
                if !is_first_time {
                    println!("");
                }
                is_first_time = false;
                if file_count > 1 {
                    println!("==> {} <==", &filename);
                }

                match config.bytes {
                    Some(c) => {
                        let mut buffer = vec![0; c as usize];
                        let bytes_read = file.read(&mut buffer)?;
                        let string = String::from_utf8_lossy(&buffer[..bytes_read]);
                        print!("{}", string);
                    }
                    _ => {
                        let mut line_count = 1;
                        let mut line = String::new();
                        while file.read_line(&mut line)? > 0 {
                            if line_count > config.lines {
                                break;
                            }
                            line_count += 1;
                            print!("{}", line);
                            line.clear();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
