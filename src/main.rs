use std::{fs::{self, ReadDir}, io, time::SystemTime};

use std::io::Read;
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use flate2::read::GzDecoder;

fn main() {
    println!("Please enter the path of the instances or log directory:");
    let root_dir_path = readln();
    let root_dir = fs::read_dir(root_dir_path.trim()).unwrap();
    let files = read_logs_dir(root_dir);

    println!("{}", files.last().unwrap());
}

fn read_logs_dir(dir: ReadDir) -> Vec<String> {

    // as strings with date
    let mut files: Vec<String> = Vec::new();

    for entry in dir {
        if let Ok(entry) = entry {
            if entry.file_type().is_err() {
                eprintln!("Couldn't read file type: {}", entry.path().display());
                continue;
            }
            if entry.file_type().unwrap().is_dir() {
                continue;
            }
            let time: DateTime<Local>;
            let content = if entry.file_name().to_string_lossy().ends_with(".log") {
                let s = match fs::read_to_string(entry.path()) {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("Couldn't read file: {}", entry.path().display());
                        String::new()
                    }
                };
                time = DateTime::from(match entry.metadata() {
                    Ok(m) => m.created().unwrap_or(SystemTime::UNIX_EPOCH),
                    Err(_) => SystemTime::UNIX_EPOCH
                });
                s
            } else {
                let bytes = fs::read(entry.path()).unwrap();
                let mut gz = GzDecoder::new(&bytes[..]);
                let mut s = String::new();
                gz.read_to_string(&mut s).unwrap();
                time = if entry.file_name().to_string_lossy().len() > 10 {
                    match NaiveDate::parse_from_str(&entry.file_name().to_string_lossy()[..10], "%Y-%m-%d") {
                        Ok(t) => t.and_time(NaiveTime::from_hms_opt(0,0,0).unwrap()).and_local_timezone(Local).unwrap(),
                        Err(_) => {
                            eprintln!("Couldn't read parse date: {}", entry.file_name().to_string_lossy());
                            DateTime::from(SystemTime::UNIX_EPOCH)
                        }
                    }
                } else {
                    DateTime::from(SystemTime::UNIX_EPOCH)
                };
                s
            };
            
            let date_string = time.date_naive().to_string();
            let mut date_content = String::new();
            for line in content.split('\n') {
                date_content.push_str(&format!("[{}] {}\n", date_string, line));
            }
            files.push(date_content);
        }
    }
    return files;
}

fn readln() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    return input;
}
