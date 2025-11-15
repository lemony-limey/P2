//! File Tree Browser
//! (based on Saleem's java variant)

mod configuration;
mod soft_state;

use anyhow::Context;
use configuration::Configuration;
use log::{debug, info, warn};
use parser::{
    protocol::{
        identifier::{FQDN, Identifier, UID},
        message::{Header, Message, Payload, Timestamp},
        serial_number::SerialNumber,
        search::SearchMessage
    }
};
use std::{
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration
};
use std::io::stdin;
use tokio::sync::mpsc::Receiver;

const PROPERTIES_FILE: &str = "filetreebrowser.toml";


pub struct FileTreeBrowser
{
    pub this_dir: PathBuf,
    root_path: PathBuf,
    pub config: Configuration,
}

impl FileTreeBrowser 
{
    pub fn new(path: impl AsRef<Path>, root_path: PathBuf, config: Configuration) -> Self 
    {
        debug!("Creating FileTreeBrowser for path: {:?}", path.as_ref());
        Self
        {
            this_dir: path.as_ref().to_path_buf(),
            root_path,
            config,
        }
    }

    pub fn logical_path_name(&self, path: &Path) -> String 
    {
        path.strip_prefix(&self.root_path)
            .ok()
            .and_then(|p| p.to_str())
            .map(|s| if s.is_empty() { "/" } else { s })
            .unwrap_or("/")
            .to_string()
    }

    /// print a formatted list of files and directories.
    pub fn print_list(&self) -> anyhow::Result<()>
    {
        let entries = fs::read_dir(&self.this_dir)?;

        println!("\n+++  id: {}", self.config.uid);
        println!("+++ dir: {}", self.logical_path_name(&self.this_dir));
        println!("+++\tfilename:");

        let mut count = 0;
        for (i, entry) in entries.enumerate() 
        {
            if let Ok(entry) = entry 
            {
                let path = entry.path();
                let mut name = entry.file_name().to_string_lossy().to_string();

                if path.is_dir()
                {
                    name.push(std::path::MAIN_SEPARATOR);
                }

                println!("{}\t{}", i, name);
                count += 1;
            }
        }

        println!("+++");
        debug!("Listed {} entries in directory", count);
        Ok(())
    }

    /// search for a file or directory by name in the current directory.
    pub fn search_list(&self, name: &str) -> Option<PathBuf>
    {
        debug!("Searching for: {}", name);
        if let Ok(entries) = fs::read_dir(&self.this_dir) 
        {
            for entry in entries.flatten()
            {
                if entry.file_name().to_string_lossy() == name 
                {
                    debug!("Found: {:?}", entry.path());
                    return Some(entry.path());
                }
            }
        }
        warn!("Not found: {}", name);
        None
    }

    pub fn get_parent(&self) -> Option<PathBuf> {
        self.this_dir.parent().map(|p| p.to_path_buf())
    }
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S%.3f").to_string()
}

fn display_help()
{
    let lines = [
        "--* Welcome to the simple FileTreeBrowser. *--",
        "* The display consists of:",
        "\t- The name of the current directory",
        "\t- The list of files (the numbers for the files are of no",
        "\t  significance, but may help you with debugging).",
        &format!(
            "* Files that are directories have trailing '{}'.",
            std::path::MAIN_SEPARATOR
        ),
        "* Use text entry to navigate the directory tree.",
        "\t.\t\tTo refresh the view of the current directory.",
        "\t..\t\tTo move up a directory level.",
        "\tfilename\tTo list file details (if it is a file) or to",
        "\t\t\tmove into that directory (if it is a directory name).",
        "\t:services\tTo list the services offered.",
        "\t:nodes\t\tTo list the other nodes discovered.",
        "\t:download\tTo download a file.",
        "\t:quit\t\tTo quit the program.",
        "\t:help\t\tTo print this message.",
    ];

    for line in &lines 
    {
        println!("{}", line);
    }
}

fn display_services(config: &Configuration) 
{
    println!(
        ":id={}:timestamp={}:search={},download={}:",
        config.uid,
        timestamp(),
        config.search,
        config.download
    );
}

/// TODO: Requirement 1
/// Using discovery multicast messages, find all 
fn nodes(config: &mut Configuration) -> anyhow::Result<()>
{
    println!("\n * nodes: to do :-)");

    // Query storage


    // Print all displayed
    // println!("");

    Ok(())
}

/// TODO: Requirement 2
fn search(config: &mut Configuration) -> anyhow::Result<()>
{
    let mut search_string = String::new();
    stdin().lock().read_line(&mut search_string)
        .with_context(|| "search(): could not get search string from stdin")?;

    let serial_number = SerialNumber(config.get_next_serial_number());
    let message = Message::new(
        Header::new(
            Identifier::new(
                UID(config.uid.to_string()),
                FQDN(config.fqdn.to_string()),
            ),
            serial_number,
            Timestamp::now()
                .with_context(|| "nodes(): could not create timestamp with supplied values")?,
        ),
        Payload::Search(
            SearchMessage::Request { search_string }
        )
    );

    // Send message


    Ok(())
}

/// TODO: Requirement 3
fn download(config: &mut Configuration) -> anyhow::Result<()>
{
    println!("\n * download: to do :-)");

    Ok(())
}

fn read_line() -> anyhow::Result<String>
{
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

pub async fn run(
    rx_beacon_messages: Receiver<Message>,
) -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let mut config = Configuration::load(PROPERTIES_FILE)?;

    println!("port: {}, ipv6 addr: {}", config.m_port, config.m_addr6);

    info!("Starting FileTreeBrowser");
    info!("Configuration loaded: id={}, root_dir={}", config.uid, config.root_dir);
    debug!("Full config: {:?}", config);

    let root_path = fs::canonicalize(&config.root_dir)?;
    let mut ftb = FileTreeBrowser::new(&root_path, root_path.clone(), config.clone());

    ftb.print_list()?;

    let mut quit_browser = false;

    while !quit_browser 
    {
        print!("\n[filename | '.' | '..' | ':services' | ':nodes' | ':search' | ':download' | ':quit' | ':help'] ");
        io::stdout().flush()?;

        let user_cmd = loop
        {
            if let Ok(line) = read_line() 
            {
                if !line.is_empty() 
                {
                    break line;
                }
            }
            tokio::time::sleep(Duration::from_millis(ftb.config.sleep_time)).await;
        };

        debug!("User command: {}", user_cmd);

        match user_cmd.to_lowercase().as_str() 
        {
            ":quit" => {
                info!("Quitting application");
                quit_browser = true;
            }

            ":help" => display_help(),

            ":services" => display_services(&ftb.config),

            "." => ftb.print_list()?,

            ".." => {
                if ftb.this_dir == root_path 
                {
                    println!("At root : cannot move up.\n");
                    warn!("Attempt to move above root directory");
                } 
                else if let Some(parent) = ftb.get_parent() 
                {
                    println!("<<< {}\n", ftb.logical_path_name(&parent));
                    info!("Moving up to: {}", ftb.logical_path_name(&parent));
                    ftb = FileTreeBrowser::new(parent, root_path.clone(), ftb.config.clone());
                }
            }

            ":nodes" => {
                if let Err(_) = nodes(&mut config) { quit_browser = true; }
            },

            ":search" => {
                if let Err(_) = search(&mut config) { quit_browser = true; }
            },

            ":download" => {
                if let Err(_) = download(&mut config) { quit_browser = true; }
            },

            _ => {
                if let Some(path) = ftb.search_list(&user_cmd) 
                {
                    if path.is_file() 
                    {
                        println!("file: {}", ftb.logical_path_name(&path));
                        if let Ok(metadata) = fs::metadata(&path) 
                        {
                            println!("size: {}", metadata.len());
                            info!(
                                "File accessed: {} (size: {})",
                                ftb.logical_path_name(&path),
                                metadata.len()
                            );
                        }
                    } 
                    else if path.is_dir() 
                    {
                        println!(">>> {}", ftb.logical_path_name(&path));
                        info!("Entering directory: {}", ftb.logical_path_name(&path));
                        ftb = FileTreeBrowser::new(path, root_path.clone(), ftb.config.clone());
                    }
                } 
                else 
                {
                    println!("Unknown command or filename: '{}'", user_cmd);
                    warn!("Unknown command or filename: '{}'", user_cmd);
                }
            }
        }
    }

    info!("FileTreeBrowser terminated");
    Ok(())
}
