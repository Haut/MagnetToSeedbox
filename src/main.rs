mod magnet;

use clap::{App, Arg};
use crate::magnet::{send_remote_file, get_host_info, convert_magnet};

fn args() -> App<'static, 'static> {
    App::new("Magnet")
        .version("0.1.0")
        .about("CLI tool to send magnet links to seedboxes")
        .usage("<MAGNET_LINK> [--config <FILE> | --host <HOST> --user <USER> --pass <PASS> --dir <DIR>]")
        .arg(Arg::with_name("magnet_link")
            .takes_value(true)
            .value_name("MAGNET_LINK")
            .help("Magnet Link to transform into .torrent and send to host")
            .required(true))
        .arg(Arg::with_name("config")
            .long("config")
            .short("c")
            .takes_value(true)
            .value_name("FILE")
            .help("Config file [default: config.yaml]")
            .conflicts_with_all(&["host", "user", "pass", "dir"]))
        .arg(Arg::with_name("host")
            .long("host")
            .short("h")
            .takes_value(true)
            .value_name("HOST")
            .help("SFTP host")
            .requires_all(&["user", "pass", "path"]))
        .arg(Arg::with_name("user")
            .long("user")
            .short("u")
            .takes_value(true)
            .value_name("USER")
            .help("SFTP username")
            .requires_all(&["host", "pass", "path"]))
        .arg(Arg::with_name("pass")
            .long("pass")
            .short("p")
            .takes_value(true)
            .value_name("PASS")
            .help("SFTP password")
            .requires_all(&["host", "user", "path"]))
        .arg(Arg::with_name("dir")
            .long("dir")
            .short("d")
            .takes_value(true)
            .value_name("DIR")
            .help("SFTP directory")
            .requires_all(&["host", "user", "pass"]))
}

//@TODO add actual error handling
fn main() {
    match get_host_info(args().get_matches()) {
        Ok(arguments) => match convert_magnet(&*arguments.dir, &*arguments.link) {
            Ok(file_info) => match send_remote_file(&*arguments.host, &*arguments.user, &*arguments.pass, &file_info.name, &file_info.data) {
                Ok(_) => println!("Sent the following to {}@{}\nFilename: {}\nData: {}", &*arguments.user, &*arguments.host, &file_info.name, &file_info.data),
                Err(e) => eprintln!("{:?}", e)
            },
            Err(e) => eprintln!("{:?}", e)
        },
        Err(e) => eprintln!("{:?}", e)
    }
}
