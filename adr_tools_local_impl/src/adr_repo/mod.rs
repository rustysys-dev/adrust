extern crate slog;
extern crate slog_term;
use slog::*;

use std::fs::{self, File};
use std::io::{self};
use std::io::prelude::*;
use std::path::Path;

extern crate regex;
use regex::Regex;

use walkdir::{DirEntry, WalkDir};

extern crate adr_config;
use adr_config::config::AdrToolConfig;

fn get_logger() -> slog::Logger {
    let cfg: AdrToolConfig = adr_config::config::get_config();
    
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter::new(drain, Level::from_usize(cfg.log_level).unwrap_or(Level::Debug)).fuse();

    let log = slog::Logger::root(drain, o!());

    log
}

///
/// Creates the file (based on template file). Returns true if file is created, false if not 
/// (because target file already exists...)
pub fn create_adr(name: &str, templates_dir: &Path, src_dir: &Path) -> io::Result<(bool)> {
    let name = match format_decision_name(name) {
        Ok(name) => name,
        Err(_why) => panic!(format!("Problem while formatting name [{}]", name)),
    };
    let target_path = src_dir.join(format!("{}.adoc", name));
    let is_target_file = target_path.is_file();
    if !is_target_file {
        let path_to_template = templates_dir.join("adr-template-v0.1.adoc");
        match path_to_template.exists() {
            true => {
                fs::copy(path_to_template, &target_path)?;
                info!(get_logger(), "New ADR {:?} created", target_path);
            },
            false => {
                error!(get_logger(), "[{}] was not found", "adr-template-v0.1.adoc" );
            }
        }
    }
    else {
        error!(get_logger(), "Decision already exists. Please use another name", );
    }

    Ok(!is_target_file)
}

fn extract_seq_id(name: &str) -> Result<(usize)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d+)").unwrap();
    }

    let cap = match RE.captures(name) {
        Some(val) => val, 
        None => {
            error!(get_logger(), "Unable to extract_seq_id from [{}]", name);
            panic!();
        },
    };

    debug!(get_logger(), "found first match [{}]", cap[0].to_string());
    let id: usize = cap[0].to_string().parse().unwrap();

    Ok(id)
}

pub fn format_decision_name(name: &str) -> Result<(String)> {
    let name = name.to_ascii_lowercase();
    let name = name.replace(" ", "-");

    Ok(name.to_string())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn list_all_adr(dir: &str) -> io::Result<(Vec<String>)> {
    let mut results = std::vec::Vec::new();

    if Path::new(dir).is_dir() {
        let walker = WalkDir::new(dir).into_iter();
        for entry in walker.filter_entry( |e| !is_hidden(e) ) {
            let entry = entry?;
            let metadata = entry.metadata().unwrap();
            if metadata.is_file() {
                let path = entry.path().display();
                results.push(format!("{}", &path));
            }
        }        
    }

    Ok(results)
}

pub fn update_to_decided(adr_name: &str) -> io::Result<(bool)> {
    let mut f = File::open(adr_name)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();

    let contains = contents.contains("{cl-wip}");
    if contains {
        let new_content = contents.replace("{cl-wip}", "{cl-decided}");
        fs::write(adr_name, new_content)?;
        info!(get_logger(), "Decision Record [{}] has been decided - Congrats!!", adr_name);
    }
    else {
        error!(get_logger(), "Decision Record [{}] has certainly not the right status and cannot be updated", adr_name);
    }

    Ok(contains)
}

pub fn superseded_by(adr_name: &str, by: &str) -> io::Result<()> {
    //manage the adr_name
    let mut contents = String::new();
    {
        let mut f = File::open(adr_name)?;
        f.read_to_string(&mut contents).unwrap();
    }
    let superseded_by = format!("{{cl-superseded}} {}", by);
    let new_content = contents.replace("{cl-decided}", &superseded_by);
    fs::write(adr_name, new_content)?;

    //manage the by
    let mut contents = String::new();
    {
        let mut f = File::open(by)?;
        f.read_to_string(&mut contents).unwrap();
    }
    let supersed = format!("{{cl-supersedes}} {}", adr_name);
    let new_content = contents.replace("{cl-decided}", &supersed);
    fs::write(by, new_content)?;

    info!(get_logger(), "Decision Record [{}] has been superseded by [{}]", adr_name, by);

    Ok(())
}

pub fn completed_by(_adr_name: &str, _by: &str) -> io::Result<()> {
    println!("et hops depuis un autre crate");

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_seq() {
        let seq = super::extract_seq_id("01-my-decision.adoc").unwrap();
        assert_eq!(seq, 1);
        let seq = super::extract_seq_id("00000010-my-decision.adoc").unwrap();
        assert_eq!(seq, 10);
        let seq = super::extract_seq_id("mypath/00000001-my-decision.adoc").unwrap();
        assert_eq!(seq, 1);
        let seq = super::extract_seq_id("mypath/00000001-my-decision-594.adoc").unwrap();
        assert_eq!(seq, 1);
        let seq = super::extract_seq_id("mypath/00000001-my-decision-594-full.adoc").unwrap();
        assert_eq!(seq, 1);
        let seq = super::extract_seq_id("00000001-my-decision-594-full.adoc").unwrap();
        assert_eq!(seq, 1);
        let seq = super::extract_seq_id("mypath/00000001/00000002-my-decision-594-full.adoc").unwrap();
        assert_eq!(seq, 1);

        let result = std::panic::catch_unwind(|| super::extract_seq_id("path/my-decision-full.adoc"));
        assert!(result.is_err());
    }

    #[test]
    fn test_format_decision_name() {
        let name = super::format_decision_name("my-decision").unwrap();
        assert_eq!(name, "my-decision");
        let name = super::format_decision_name("my decision").unwrap();
        assert_eq!(name, "my-decision");
        let name = super::format_decision_name("my Decision").unwrap();
        assert_eq!(name, "my-decision");
    }
}