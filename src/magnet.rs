use ssh2::{Session, Channel};
use std::error::Error;
use std::net::TcpStream;
use std::path::Path;
use std::io::Write;
use yaml_rust::{YamlLoader, Yaml};
use std::fs;
use regex::Regex;
use clap::ArgMatches;

fn create_session(host: &str) -> Result<Session, Box<dyn Error>> {
    match (TcpStream::connect(host), Session::new()) {
        (Ok(tcp), Ok(mut sess)) => {
            sess.set_tcp_stream(tcp);
            return Ok(sess);
        },
        (Err(_), _) => Err("Could not initialize TcpStream.".into()),
        (_, Err(_)) => Err("Could not create ssh Session.".into()),
    }
}

fn handshake(mut sess: Session) -> Result<Session, Box<dyn Error>> {
    match sess.handshake() {
        Ok(_) => Ok(sess),
        Err(_) => Err("Unable to successfully handshake.".into())
    }
}

fn authenticate(sess: Session, username: &str, password: &str) -> Result<Session, Box<dyn Error>> {
    match sess.userauth_password(username, password) {
        Ok(_) => Ok(sess),
        Err(_) => Err("Incorrect login credentials.".into())
    }
}

fn add_file(sess: Session, filename: &str, data_len: u64) -> Result<Channel, Box<dyn Error>> {
    match sess.scp_send(Path::new(filename), 0o666, data_len, None) {
        Ok(remote_file) => Ok(remote_file),
        Err(_) => Err("Could not create remote file.".into())
    }
}

fn write_file(mut remote_file: Channel, data: &str) -> Result<usize, Box<dyn Error>> {
    match remote_file.write(data.as_bytes()) {
        Ok(written) => Ok(written),
        Err(_) => Err("Could not write to remote file.".into())
    }
}

pub fn send_remote_file(host: &str, user: &str, pass: &str, filename: &str, data: &str) -> Result<usize, Box<dyn Error>> {
    create_session(host)
        .and_then(handshake)
        .and_then(|sess| authenticate(sess, user, pass))
        .and_then(|sess| add_file(sess, filename, data.len() as u64))
        .and_then(|remote_file| write_file(remote_file, data))
}

pub struct FileInfo {
    pub name: String,
    pub data: String }

pub struct Arguments {
    pub link: String,
    pub host: String,
    pub user: String,
    pub pass: String,
    pub dir: String }

fn to_file_info(path: &str, hash: &str, magnet: &str) -> FileInfo {
    FileInfo {
        name: format!("{}/meta-{}.torrent", path, hash),
        data: format!("d10:magnet-uri{}:{}e", magnet.len().to_string(), magnet)}
}

pub fn convert_magnet(path: &str, magnet: &str) -> Result<FileInfo, Box<dyn Error>> {
    match Regex::new(r"^magnet:\?xt=urn:btih:([a-z,0-9]+)").unwrap().captures(magnet) {
        Some(hash) => Ok(to_file_info(path, &hash[1], magnet)),
        None => Err("Input must match '^magnet:\\?xt=urn:btih:[a-z,0-9]+.*'".into()),
    }
}

fn file_to_string(file: &str) -> Result<Yaml, Box<dyn Error>> {
    match fs::read_to_string(file) {
        Ok(contents) => match YamlLoader::load_from_str(&*contents) {
            Ok(a) => Ok(a[0].to_owned()),
            Err(_) => Err("Could not load yaml from file.".into()),
        },
        Err(_) => Err("Could not read file to string.".into()),
    }
}

fn to_argument(link: &str, host: &str, user: &str, pass: &str, dir: &str) -> Arguments {
    Arguments {
        link: link.to_string(),
        host: host.to_string(),
        user: user.to_string(),
        pass: pass.to_string(),
        dir: dir.to_string()
    }
}

fn argument_from_config(arg_matches: ArgMatches, doc: Yaml) -> Result<Arguments, Box<dyn Error>> {
    if let (Some(link), Some(host), Some(user), Some(pass), Some(dir)) = (
        arg_matches.value_of("magnet_link"),
        doc["host"].as_str(),
        doc["user"].as_str(),
        doc["pass"].as_str(),
        doc["dir"].as_str()) {
        Ok(to_argument(link, host, user, pass, dir))
    } else {
        Err("Could not read config from file. Needs host, user, pass, and path.".into())
    }
}

fn arguments_from_explicit(arg_matches: ArgMatches) -> Result<Arguments, Box<dyn Error>> {
    if let (Some(link), Some(host), Some(user), Some(pass), Some(dir)) = (
        arg_matches.value_of("magnet_link"),
        arg_matches.value_of("host"),
        arg_matches.value_of("user"),
        arg_matches.value_of("pass"),
        arg_matches.value_of("dir")) {
        Ok(to_argument(link, host, user, pass, dir))
    } else {
        Err("Could not read arguments.".into())
    }
}

pub fn get_host_info(arg_matches: ArgMatches) -> Result<Arguments, Box<dyn Error>> {
    if arg_matches.is_present("config") || !arg_matches.is_present("host") {
        file_to_string(arg_matches.value_of("config").unwrap_or("config.yaml"))
            .and_then(|doc| argument_from_config(arg_matches, doc))
    } else {
        arguments_from_explicit(arg_matches)
    }
}