use chrono::{DateTime, Utc};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;

/// Program which will split your single long log into multiple log
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// output directory of log file
    #[arg(short, long)]
    out_dir: PathBuf,

    /// temporary directory which will store current
    /// log file
    #[arg(short, long)]
    tmp_dir: Option<PathBuf>,
}

fn get_file_path(mut dir: PathBuf, time: &DateTime<Utc>) -> PathBuf {
    dir.push(time.format("%Y-%m-%d-%H-%M").to_string());
    dir
}

fn main() {
    let args = Args::parse();

    let mut start_time = Utc::now();

    let log_dir = args.out_dir;
    let out_dir = if let Some(x) = args.tmp_dir.as_ref() {
        x.clone()
    } else {
        log_dir.clone()
    };
    let move_after_close = out_dir != log_dir;
    let mut file_path = get_file_path(out_dir.clone(), &start_time);

    let file = File::create(&file_path).expect("Unable to open file");

    let reader = std::io::stdin().lock();
    let mut writer = BufWriter::new(file);

    let interval = chrono::Duration::minutes(1);

    reader.lines().into_iter().for_each(|line| {
        let line = match line {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Error reading from pipe: {e:?}");
                return;
            }
        };

        let current_time = Utc::now();
        let duration = current_time.signed_duration_since(start_time);

        if duration > interval {
            println!("Starting new file");
            start_time = current_time;

            let new_path = get_file_path(out_dir.clone(), &start_time);
            println!("New file: {new_path:?}");
            if move_after_close {
                match std::fs::rename(&file_path, log_dir.join(file_path.file_name().unwrap())) {
                    Ok(..) => {
                        println!(
                            "move from {file_path:?} to {:?}",
                            log_dir.join(file_path.file_name().unwrap())
                        );
                    }
                    Err(e) => {
                        eprintln!("Cannot move out of tmp dir: {e:?}");
                    }
                }
            }
            file_path = new_path;

            let file = File::create(&file_path).expect("Unable to open file");
            writer = BufWriter::new(file);
        }

        match write!(writer, "{line}\n") {
            Ok(..) => {}
            Err(e) => {
                eprintln!("Unable to write log file: {e:?}");
            }
        };
        match writer.flush() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Error flushing to log file: {e:?}, continuing without flushing");
            }
        };
    });
    if move_after_close {
        match std::fs::rename(&file_path, log_dir.join(file_path.file_name().unwrap())) {
            Ok(..) => {
                println!(
                    "move from {file_path:?} to {:?}",
                    log_dir.join(file_path.file_name().unwrap())
                );
            }
            Err(e) => {
                eprintln!("Cannot move out of tmp dir: {e:?}");
            }
        }
    }
}
