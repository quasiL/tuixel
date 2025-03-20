use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

use crate::components::users::User;

pub fn get_users_from_passwd(doc_root_prefix: &str) -> Vec<User> {
    let path = Path::new("/etc/passwd");
    let file = File::open(path);

    let file = match file {
        Ok(f) => f,
        Err(e) => {
            return vec![User {
                username: format!("Error: {}", e),
                docroot: String::new(),
                shell: String::new(),
            }];
        }
    };

    let reader = io::BufReader::new(file);
    let mut users = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                return vec![User {
                    username: format!("Error reading line: {}", e),
                    docroot: String::new(),
                    shell: String::new(),
                }];
            }
        };

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            continue;
        }

        let username = parts[0];
        let home_dir = parts[5];
        let shell = parts[6];

        if home_dir.starts_with(doc_root_prefix) {
            users.push(User {
                username: username.to_string(),
                docroot: home_dir.to_string(),
                shell: shell.to_string(),
            });
        }
    }

    if users.is_empty() {
        users.push(User {
            username: "No users found".to_string(),
            docroot: String::new(),
            shell: String::new(),
        });
    }

    users
}

pub fn constraint_len_calculator(items: &[User]) -> (u16, u16, u16) {
    let username_len = items
        .iter()
        .map(|user| user.username.as_str())
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let docroot_len = items
        .iter()
        .map(|user| user.docroot.as_str())
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let shell_len = items
        .iter()
        .map(|user| user.shell.as_str())
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (username_len as u16, docroot_len as u16, shell_len as u16)
}
