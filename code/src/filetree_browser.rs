//! File Tree Browser
//! (based on Saleem's java variant)

pub mod configuration;

use crate::soft_state::Node;
use crate::soft_state::FTBSoftState;
use tokio::{
    sync::RwLock,
    fs,
    io::AsyncWriteExt
};
use std::{
    sync::Arc,
    io::{
        self,
        // BufRead,
        stdin
    },
    path::{Path, PathBuf},
    time::Duration,
};
use std::io::BufRead;
use anyhow::Context;
use configuration::FiletreeBrowserConfiguration;
use log::{debug, info, warn};
use crate::{
    message_socket::MessageSocket,
    protocol::{
        identifier::{Identifier, FQDN, UID},
        message::{Header, Message, Payload, Timestamp},
        search::SearchMessage,
        serial_number::SerialNumber,
    },
};

const PROPERTIES_FILE: &str = "filetreebrowser.toml";


pub struct FileTreeBrowser
{
    root_path: PathBuf,
    pub this_dir: PathBuf,
    pub config: FiletreeBrowserConfiguration,
    socket: Arc<RwLock<MessageSocket>>,
    storage: Arc<RwLock<FTBSoftState>>,
}

impl FileTreeBrowser
{
    pub fn new(
        path: impl AsRef<Path>,
        root_path: PathBuf,
        config: FiletreeBrowserConfiguration,
        socket: Arc<RwLock<MessageSocket>>,
        storage: Arc<RwLock<FTBSoftState>>,
    ) -> Self
    {
        debug!("Creating FileTreeBrowser for path: {:?}", path.as_ref());
        Self
        {
            this_dir: path.as_ref().to_path_buf(),
            root_path,
            config,
            socket,
            storage,
        }
    }

    pub async fn logical_path_name(&self, path: &Path) -> anyhow::Result<String>
    {
        // This data has to be cloned since it doesn't have a static lifetime (&self and &path
        // could hypothetically be dropped before this task finishes).
        let path = path.to_owned();
        let root_path = &self.root_path;
        let root_path = root_path.to_owned();

        let result = tokio::task::spawn_blocking(move || {
            path.strip_prefix(root_path)
                .ok()
                .and_then(|p| p.to_str())
                .map(|s| if s.is_empty() { "/" } else { s })
                .unwrap_or("/")
                .to_string()
        }).await?;

        Ok(result)
    }

    /// print a formatted list of files and directories.
    pub async fn print_list(&self) -> anyhow::Result<()>
    {
        let mut entries = fs::read_dir(&self.this_dir).await?;

        println!("\n+++  id: {}", self.config.uid);
        println!("+++ dir: {}", self.logical_path_name(&self.this_dir).await?);
        println!("+++\tfilename:");

        let mut count = 0;
        while let Ok(Some(entry)) = entries.next_entry().await
        {
            let path = entry.path();
            let mut name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir()
            {
                name.push(std::path::MAIN_SEPARATOR);
            }

            println!("{}\t{}", count, name);
            count += 1;
        }

        println!("+++");
        debug!("Listed {} entries in directory", count);
        Ok(())
    }

    /// search for a file or directory by name in the current directory.
    /// TODO: Return a Vec of results?
    pub async fn search_list(&self, name: &str) -> anyhow::Result<Option<PathBuf>>
    {
        debug!("Searching for: {}", name);
        let mut entries = fs::read_dir(&self.this_dir).await?;
        while let Some(entry) = entries.next_entry().await?
        {
            if entry.file_name().to_string_lossy() == name
            {
                debug!("Found: {:?}", entry.path());
                return Ok(Some(entry.path()));
            }
        }
        warn!("Not found: {}", name);
        Ok(None)
    }

    pub fn get_parent(&self) -> Option<PathBuf>
    {
        self.this_dir.parent().map(|p| p.to_path_buf())
    }
}

fn timestamp() -> String
{
    chrono::Utc::now().format("%Y%m%d-%H%M%S%.3f").to_string()
}

async fn display_help() -> anyhow::Result<()>
{
    tokio::task::spawn_blocking(move || {
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
    }).await?;

    Ok(())
}

fn display_services(config: &FiletreeBrowserConfiguration)
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
/// Using discovery multicast messages, find all of the nodes on the network.
/// Print them out.
async fn nodes(
    ftb: &mut FileTreeBrowser,
) -> anyhow::Result<()>
{
    println!("\n * nodes: to do :-)");

    // Query storage


    // Print all displayed
    // println!("");

    Ok(())
}

/// TODO: Requirement 2
/// We don't return any data back to the main program, we just print the files to stdout.
async fn search(
    ftb: &mut FileTreeBrowser,
) -> anyhow::Result<()>
{
    // Get the search string from the user.
    let target_file = read_line().await?;
    let target_file = target_file.as_str();

    // Query storage and check to see if we already know who has what file.
    if let Some(node_with_file) = ftb.storage.read().await.find_file(target_file)
    {
        println!()
    }

    // If storage doesn't have what we need, send out a message
    let mut search_string = String::new();
    {
        stdin().lock().read_line(&mut search_string)
            .with_context(|| "search(): could not get search string from stdin")?;
    }

    let serial_number = SerialNumber(ftb.config.next_download_serial_number());
    let message = Message::new(
        Header::new(
            Identifier::new(
                UID(ftb.config.uid.to_string()),
                FQDN(ftb.config.fqdn.to_string()),
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
    {
        ftb.socket.read().await.tx_message(message).await?;
    }

    // Wait for received messages
    tokio::time::sleep(Duration::from_millis(500)).await;

    Ok(())
}

/// TODO: Requirement 3
/// Send a
async fn download(
    ftb: &mut FileTreeBrowser,
) -> anyhow::Result<()>
{
    println!("\n * download: to do :-)");

    Ok(())
}

async fn read_line() -> anyhow::Result<String>
{
    let result = tokio::task::spawn_blocking(move || {
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        Ok(line.trim().to_string())
    }).await;

    result?
}

pub async fn run_ftb(
    ftb: &mut FileTreeBrowser,
) -> anyhow::Result<()>
{
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
    //     .format_timestamp_millis()
    //     .init();

    let mut config = FiletreeBrowserConfiguration::load(PROPERTIES_FILE)?;

    println!("port: {}, ipv6 addr: {}", config.m_port, config.m_addr6);

    info!("Starting FileTreeBrowser");
    info!("Configuration loaded: id={}, root_dir={}", config.uid, config.root_dir);
    debug!("Full config: {:?}", config);

    ftb.print_list().await?;

    let mut quit_browser = false;

    while !quit_browser
    {
        print!("\n[filename | '.' | '..' | ':services' | ':nodes' | ':search' | ':download' | ':quit' | ':help'] ");
        tokio::io::stdout().flush().await?;

        let user_cmd = loop
        {
            if let Ok(line) = read_line().await
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

            ":help" => display_help().await?,

            ":services" => display_services(&ftb.config),

            "." => ftb.print_list().await?,

            ".." => {
                if ftb.this_dir == ftb.root_path
                {
                    println!("At root : cannot move up.\n");
                    warn!("Attempt to move above root directory");
                }
                else if let Some(parent) = ftb.get_parent()
                {
                    println!("<<< {}\n", ftb.logical_path_name(&parent).await?);
                    info!("Moving up to: {}", ftb.logical_path_name(&parent).await?);
                }
            }

            ":nodes" => nodes(ftb).await?,

            ":search" => search(ftb).await?,

            ":download" => download(ftb).await?,

            _ => {
                if let Some(path) = ftb.search_list(&user_cmd).await?
                {
                    if path.is_file()
                    {
                        println!("file: {}", ftb.logical_path_name(&path).await?);
                        if let Ok(metadata) = fs::metadata(&path).await
                        {
                            println!("size: {}", metadata.len());
                            info!(
                                "File accessed: {} (size: {})",
                                ftb.logical_path_name(&path).await?,
                                metadata.len()
                            );
                        }
                    }
                    else if path.is_dir()
                    {
                        println!(">>> {}", ftb.logical_path_name(&path).await?);
                        info!("Entering directory: {}", ftb.logical_path_name(&path).await?);
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
