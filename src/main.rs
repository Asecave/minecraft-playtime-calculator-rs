use std::{fs::{self, ReadDir}, io, path::Path, time::SystemTime};

use std::io::Read;
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use flate2::read::GzDecoder;

fn main() {
    println!("Please enter the path of the instances or log directory:");
    let root_dir_path = "/home/asecave/.local/share/ATLauncher/instances/";//readln();
    let mut log_dirs: Vec<ReadDir> = Vec::new();

    find_log_dirs(&Path::new(root_dir_path.trim()), &mut log_dirs);

    let mut files = Vec::new();

    for log_dir in log_dirs {
        let mut f = read_logs_dir(log_dir);
        files.append(&mut f);
    }

    let mut total_time: u64 = 0; // in seconds
    let mut times = Vec::new();

    for file in files {
        let lines: Vec<&str> = file.split('\n').collect();
        let first_line = if let Some(line) = lines.first() {line} else {continue;};
        let last_line = if let Some(line) = lines.last() {line} else {continue;};
        if first_line.len() < 22 || last_line.len() < 22 {
            continue;
        }
        let start_seconds = if let Some(s) = get_seconds(&first_line[14..22]) {s} else {continue;};
        let end_seconds = if let Some(s) = get_seconds(&last_line[14..22]) {s} else {continue;};
        
        let seconds = end_seconds - start_seconds;
        total_time += seconds;
        times.push(seconds);
        println!("{}", convert_seconds_to_human_readable(seconds));
    }

    println!();
    times.sort();
    println!("{}", times.last().unwrap());
    println!();
    println!("{}", convert_seconds_to_human_readable(total_time));
}

fn find_log_dirs(root_dir: &Path, log_dirs: &mut Vec<ReadDir>) {
    if root_dir.is_dir() {
        let read_dir = fs::read_dir(root_dir).unwrap();
        if root_dir.file_name().unwrap() == "logs" {
            log_dirs.push(read_dir);
        } else {
            for file in read_dir {
                if let Ok(d) = file {
                    find_log_dirs(d.path().as_path(), log_dirs);
                }
            }
        }
    }
}

fn convert_seconds_to_human_readable(seconds: u64) -> String {
    format!("{}:{}:{}", seconds / 3600, (seconds % 3600) / 60, seconds & 60)
}

fn get_seconds(s: &str) -> Option<u64> {
    let h: u64 = s[0..2].parse().ok()?;
    let m: u64 = s[3..5].parse().ok()?;
    let sec: u64 = s[6..8].parse().ok()?;


    Some(h * 3600 + m * 60 + sec)
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
                let mut s = Vec::new();
                if let Err(e) = gz.read_to_end(&mut s) {
                    eprintln!("Error while reading file: {} -- Skipping. {}", entry.path().to_string_lossy(), e);
                    continue;
                }
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
                String::from_utf8_lossy(&s).into_owned()
            };
            
            let date_string = time.date_naive().to_string();
            let mut date_content = String::new();
            for line in content.split('\n') {
                if line.is_empty() {
                    continue;
                }
                date_content.push_str(&format!("[{}] {}\n", date_string, line));
            }
            files.push(date_content.trim().to_string());
        }
    }
    return files;
}

fn readln() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    return input;
}
