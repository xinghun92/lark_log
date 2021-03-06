extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate chrono;

use std::env::current_dir;
use std::fs::{read_dir, rename, File, remove_file};
use std::path::{Path, PathBuf};
use std::io::{Result, BufReader, Write, BufRead};
use std::collections::HashMap;
use chrono::DateTime;

#[derive(Serialize, Deserialize)]
struct Message<'a> {
    time: &'a str,
    message: String,
    module_path: &'a str,
    file: String,
    line: u32,
    level: &'a str,
    target: &'a str,
    thread: Option<&'a str>,
    pid: u32,
    mdc: HashMap<&'a str, &'a str>
}

fn main() {
    parse_log().unwrap();

}

fn parse_log() -> Result<()> {
    const TMP: &'static str = "tmp.log";
    let dir = current_dir()?;

    for file in read_dir(&dir)? {
        let file = file?;
        if let Some(extension) = Path::new(&file.path()).extension() {
            if extension.to_str().expect("Invalid Extension") == "log" {
                let to = file.path();
                let from = dir.join(TMP);
                rename(&to, &from)?;
                parse_log_impl(&from, &to)?;
                remove_file(&from)?;
            }
        }
    }

    Ok(())
}

fn parse_log_impl(from: &PathBuf, to: &PathBuf) -> Result<()> {
    let from = File::open(from)?;
    let mut to = File::create(to)?;
    let from = BufReader::new(from);

    for line in from.lines() {
        let line = get_parsed_line(&line?);
        write!(to, "{}", line)?;
    }

    Ok(())
}

const CONTEXT_ID: &'static str = "cid";
fn get_parsed_line(line: &str) -> String {
    match serde_json::from_str::<Message>(&line) {
        Ok(line) => {
            let time = DateTime::parse_from_rfc3339(line.time).expect("Invalid time format").format("%m-%d %H:%M:%S");
            let context_id = line.mdc.get(CONTEXT_ID).unwrap_or_else(|| &"unknown");
            format!("{} {} {} {} {} {}:{} - {}\n",
                    time, context_id, line.level, line.pid, line.thread.unwrap_or("unknown"),
                    line.file, line.line, line.message)
        }
        Err(_) => {
            format!("{}\n", line)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_log() {
        let log = r#"{"time":"2018-04-01T22:38:05.302529+08:00","message":"adding \"/Users/dinghao/Library/Application Support/Lark/sdk_storage/log/fe29.log\" as \"fe29.log\" ...","module_path":"lark_logic::utils","file":"lark-logic/src/utils.rs","line":89,"level":"INFO","target":"lark_logic::utils","thread":"invoke-2","pid":33702,"mdc":{"cid":"3nBWApkDow"}}"#;
        let log = get_parsed_line(log);
        assert_eq!(log, format!("{}\n", r#"04-01 22:38:05 3nBWApkDow INFO 33702 invoke-2 lark-logic/src/utils.rs:89 - adding "/Users/dinghao/Library/Application Support/Lark/sdk_storage/log/fe29.log" as "fe29.log" ..."#));
    }

    #[test]
    fn test_windows_log() {
        let log = r#"{"time":"2018-04-04T11:08:00.656803400+08:00","message":"fetch: cmd= 0 cost= 863","module_path":"lib_net::client::fetch","file":"lib-net\\src\\client\\fetch.rs","line":229,"level":"INFO","target":"lib_net::client::fetch","thread":"t:tokio","pid":1960,"mdc":{"cid":"3nBWApkDow"}}"#;
        let log = get_parsed_line(log);
        assert_eq!(log, format!("{}\n", r#"04-04 11:08:00 3nBWApkDow INFO 1960 t:tokio lib-net\src\client\fetch.rs:229 - fetch: cmd= 0 cost= 863"#));
    }

    #[test]
    fn test_android_log() {
        let log = r#"{"time":"2018-04-08T18:27:19.845833174+08:00","message":"get message read state: message_id= \"6535957645658947847\" read_users= 14 unread_users = 2","module_path":"lark_message::messages","file":"lark-message/src/messages.rs","line":1491,"level":"DEBUG","target":"lark_message::messages","thread":"callback-0","pid":16376,"mdc":{"cid":"3nBWApkDow"}}"#;
        let log = get_parsed_line(log);
        assert_eq!(log, format!("{}\n", r#"04-08 18:27:19 3nBWApkDow DEBUG 16376 callback-0 lark-message/src/messages.rs:1491 - get message read state: message_id= "6535957645658947847" read_users= 14 unread_users = 2"#));
    }
}
