//! File Tree Browser
//! (based on Saleem's java variant)

pub mod configuration;

use log::trace;
use log::error;
use tokio::fs::{DirEntry};
use std::collections::VecDeque;
use crate::anyhow;
use std::collections::HashSet;
use crate::soft_state::FTBSoftState;
use tokio::{
    sync::RwLock,
    fs,
    io::AsyncWriteExt,
};
use std::{
    sync::Arc,
    io::{
        stdin
    },
    path::{Path, PathBuf},
    time::Duration,
};
use std::io::BufRead;
use std::num::NonZeroU64;
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
use crate::constants::FILETREE_BROWSER_CONFIGURATION_FILE;
use crate::logical_file_path::LogicalFilePath;
use crate::protocol::download::DownloadMessage;
use crate::simple_tcp::server::run_server;

pub struct FileTreeBrowser
{
    root_path: PathBuf,
    pub this_dir: PathBuf,
    pub config: FiletreeBrowserConfiguration,
    socket: MessageSocket,
    soft_state: Arc<RwLock<FTBSoftState>>,
}

impl FileTreeBrowser
{
    pub fn new(
        path: impl AsRef<Path>,
        root_path: PathBuf,
        config: FiletreeBrowserConfiguration,
        socket: MessageSocket,
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
            soft_state: storage,
        }
    }

    pub async fn absolute_path_to_logical_path(&self, path: &Path) -> anyhow::Result<LogicalFilePath>
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

        Ok(LogicalFilePath::new(result))
    }

    pub async fn logical_path_to_absolute_path(&self, logical_path: &LogicalFilePath) -> anyhow::Result<PathBuf>
    {
        let root_path = &self.root_path;
        let Some(mut root_path_str) = root_path.to_str() else {
            return Err(anyhow!("provided file path does not exist: {:#?}", root_path));
        };
        // Logical file paths always begin with a "/", so remove a trailing "/" if one exists
        // to avoid them being doubled up.
        if let Some(result) = root_path_str.strip_suffix("/")
        {
            root_path_str = result;
        }
        // Logical path will always at least contain "/", so we don't need to manually insert it.
        let absolute_path = format!("{}{}", root_path_str, logical_path);

        Ok(PathBuf::from(absolute_path))
    }

    /// print a formatted list of files and directories.
    pub async fn print_list(&self) -> anyhow::Result<()>
    {
        let mut entries = fs::read_dir(&self.this_dir).await?;

        println!("\n+++  id: {}", self.config.uid);
        println!("+++ pwd: {}", self.absolute_path_to_logical_path(&self.this_dir).await?.get());
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

    /// Search for a file or directory by name in the current directory.
    /// Only the file name is required, not the full logical path, because we know the current
    /// directory.
    pub async fn search_pwd_for_file(&self, file_name: &str) -> anyhow::Result<Option<PathBuf>>
    {
        debug!("Searching for: {}", file_name);
        let mut entries = fs::read_dir(&self.this_dir).await?;
        while let Some(entry) = entries.next_entry().await?
        {
            if entry.file_name().to_string_lossy() == file_name
            {
                debug!("Found: {:?}", entry.path());
                return Ok(Some(entry.path()));
            }
        }
        warn!("Not found: {}", file_name);
        Ok(None)
    }

    /// Use BFS to traverse all subdirectories.
    /// Add any substring match to a results vec, which is returned.
    pub async fn search_for_substring(&self, substring: &str) -> anyhow::Result<Vec<LogicalFilePath>>
    {
        let mut matches: Vec<LogicalFilePath> = Vec::new();
        let mut initial_entry = fs::read_dir(&self.root_path).await?;
        let mut queue: VecDeque<DirEntry> = VecDeque::new();

        while let Some(new_entry) = initial_entry.next_entry().await?
        {
            queue.push_back(new_entry);
        }

        // Recursively list all files in all directories
        while !queue.is_empty()
        {
            let entry = queue.pop_front().unwrap().path();

            // If the entry is a directory, add all of its entries to the queue also.
            if entry.is_dir()
            {
                trace!("reading dir {:?}", &entry);
                let Ok(mut new_entries) = fs::read_dir(&entry).await else {
                    continue;
                };

                while let Some(new_entry) = new_entries.next_entry().await?
                {
                    trace!("adding entry: {:#?}", new_entry);
                    queue.push_back(new_entry);
                }
            }
            else if entry.is_file()
            {
                // If the current file contains a substring match, add it to the list of matches
                if let Some(entry) = entry.to_str()
                {
                    trace!("substring: {}; current entry: {}", substring, entry);
                    if entry.contains(substring)
                    {
                        matches.push(self.absolute_path_to_logical_path(Path::new(entry)).await?);
                        debug!("search_for_substring: MATCH! substring: {}; string: {}", substring, entry);
                    }
                }
            }

            trace!("queue: {:#?}", queue);
            trace!("matches: {:#?}", matches);
        }

        debug!("matches: {:#?}", matches);

        Ok(matches)
    }

    /// Check if a specific file exists in the file tree.
    /// If it does not, a call to open() will return an error.
    pub async fn does_file_exist(&self, logical_file_path: &LogicalFilePath) -> anyhow::Result<bool>
    {
        let absolute_path = self.logical_path_to_absolute_path(logical_file_path).await?;

        // Do not allow LogicalFilePaths to store paths to directories.
        Ok(absolute_path.is_file())
    }

    pub async fn get_file_size(&self, logical_file_path: &LogicalFilePath) -> anyhow::Result<u64>
    {
        let absolute_path = self.logical_path_to_absolute_path(logical_file_path).await?;
        debug!("logical_file_path: {}", logical_file_path);
        debug!("absolute file path: {:#?}", absolute_path);
        let metadata = fs::metadata(absolute_path).await?;

        Ok(metadata.len())
    }

    pub fn get_parent(&self) -> Option<PathBuf>
    {
        self.this_dir.parent().map(|p| p.to_path_buf())
    }

    pub fn get_current_identifier(&self) -> Identifier
    {
        Identifier::new(
            UID(self.config.uid.clone()),
            FQDN(self.config.fqdn.clone()),
        )
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

async fn display_services(ftb: Arc<RwLock<FileTreeBrowser>>)
{
    let ftb_handle = ftb.read().await;
    let config = &ftb_handle.config;

    println!(
        ":id={}:timestamp={}:search={},download={}:",
        config.uid,
        timestamp(),
        config.search,
        config.download
    );
}

/// Requirement 1
/// Using discovery multicast messages, find all of the nodes on the network.
/// Print them out.
async fn nodes(
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    // Print all displayed
    debug!("nodes(): making a request to storage to print all nodes");
    ftb.read().await.soft_state.read().await.print_all_nodes().await;
    Ok(())
}

/// Requirement 2 - Search
/// We don't return any data back to the main program, we just print the files to stdout.
async fn search(
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    // Get the search string from the user.
    println!("Enter the name of the file you wish to find: ");
    let search_string = read_line().await?;

    let serial_number: SerialNumber;
    let identifier: Identifier;
    let sleep_time: u64;
    // Split into 2 blocks to minimise the time we hold the exclusive write() handle for.
    {
        let mut ftb_handle = ftb.write().await;
        serial_number = SerialNumber(ftb_handle.config.next_search_serial_number());
    }
    {
        let ftb_handle = ftb.read().await;
        identifier = Identifier::new(
            UID(ftb_handle.config.uid.to_string()),
            FQDN(ftb_handle.config.fqdn.to_string())
        );
        sleep_time = ftb_handle.config.sleep_time
    }

    // Construct the search request message
    let message = Message::new(
        Header::new(
            identifier,
            serial_number,
            Timestamp::now()?,
        ),
        Payload::Search(
            SearchMessage::Request { search_string: search_string.clone() }
        )
    );

    // Send message
    {
        let ftb_handle = ftb.read().await;
        ftb_handle.socket.tx_message(message).await?;
    }
    debug!("search request sent");

    // Wait for received messages
    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    // Query storage and check to see who has what file.
    let mut nodes_with_file = HashSet::new();
    let target_file = LogicalFilePath::new(&search_string);
    if let Some(results) = ftb.read().await.soft_state.read().await.find_file_substring_match(&target_file.to_string()).await
    {
        nodes_with_file = results;
    }

    if nodes_with_file.is_empty()
    {
        println!("No nodes are currently known to contain an exact match for \"{}\". Run \":files\" to look for partial matches", &search_string);
    }
    else
    {
        println!(
            "{} nodes are currently known to contain the file {}",
            nodes_with_file.len(),
            &search_string,
        );
        for (logical_file_path, identifier) in nodes_with_file
        {
            println!("{} | {}", logical_file_path, identifier);
        }
    }

    Ok(())
}

async fn files(
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    let ftb_handle = ftb.read().await;
    let soft_state_handle = ftb_handle.soft_state.read().await;
    let nodes_handle = soft_state_handle.nodes.read().await;

    for (identifier, node) in nodes_handle.iter()
    {
        println!("Files owned by node: {}", identifier);

        for (file_path, metadata) in node.read().await.files.iter()
        {
            println!("\t{} | {}", file_path, metadata.size);
        }

        println!("-----");
    }

    Ok(())
}

/// Requirement 3
/// Send a download request for a specific file.
async fn download(
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    let serial_number: SerialNumber;
    let user_identifier: Identifier;
    // Split into 2 blocks to minimise the time we hold the exclusive write() handle for.
    {
        let mut ftb_handle = ftb.write().await;
        serial_number = SerialNumber(ftb_handle.config.next_search_serial_number());
    }
    {
        let ftb_handle = ftb.read().await;
        user_identifier = Identifier::new(
            UID(ftb_handle.config.uid.to_string()),
            FQDN(ftb_handle.config.fqdn.to_string())
        );
    }

    // If storage doesn't have what we need, send out a message
    println!("Enter the name of the file you wish to download:");
    let mut target_file = String::new();
    {
        stdin().lock().read_line(&mut target_file)
            .with_context(|| "search(): could not get search string from stdin")?;
    }
    let target_file = LogicalFilePath::new(target_file);
    let target_file_absolute_path: PathBuf;
    {
        target_file_absolute_path = ftb.read().await.logical_path_to_absolute_path(&target_file).await?;
    }

    debug!("target file logical path: {}", target_file);
    debug!("target file absolute path: {}", target_file_absolute_path);

    // If the file already exists in the local logical directory, abandon the request
    let Ok(false) = target_file_absolute_path.try_exists() else {
        println!("Cannot download requested file, it already exists locally!");
        info!("download request abandoned: file already exists locally at the same path: \"{}\"", &target_file);
        return Ok(())
    };

    // Get the identifier of the node we wish to download the file from.
    println!("Enter the identifier of the node you wish to request this file from:");
    let mut target_identifier = String::new();
    {
        stdin().lock().read_line(&mut target_identifier)
            .with_context(|| "search(): could not get search string from stdin")?;
    }
    debug!("target identifier string: {}", &target_identifier);

    let mut identifier_components = target_identifier.split('@');
    let Some(uid) = identifier_components.next() else {
        return Err(anyhow!("Invalid identifier entered"));
    };
    let Some(fqdn) = identifier_components.next() else {
        return Err(anyhow!("Invalid identifier entered"));
    };
    let target_identifier = Identifier::new(
        UID(uid.trim().to_string()),
        FQDN(fqdn.trim().to_string()),
    );

    debug!("target identifier: {}", &target_identifier);

    // Check that the node in question contains the file we are looking for. Otherwise, abandon.
    {
        let ftb_handle = ftb.read().await;
        // let logical_file_path = ftb.read().await.absolute_path_to_logical_path(target_file_absolute_path)
        let soft_state_handle = ftb_handle.soft_state.read().await;
        let Some(target_node) = soft_state_handle.get_node(&target_identifier).await else {
            info!("download abandoned: requested node \"{}\" does not exist", &target_identifier);
            return Err(anyhow!("node does not exist"));
        };
        // If node does not contain file, abandon download.
        if !target_node.read().await.files.contains_key(&target_file)
        {
            info!(
                "download abandoned: requested node \"{}\" does not contain file \"{}\"",
                &target_identifier,
                &target_file,
            );
            return Err(anyhow!("node does not contain file"));
        }
    }

    let message = Message::new(
        Header::new(
            user_identifier.clone(),
            serial_number.clone(),
            Timestamp::now()?,
        ),
        Payload::Download(
            DownloadMessage::Request {
                identifier: target_identifier,
                logical_file_path: target_file.clone(),
            },
        ),
    );

    // Send download request message
    {
        ftb.read().await.socket.tx_message(message).await?;
    }

    // Start server and report what happens
    if let Err(e) = run_server(target_file_absolute_path).await
    {
        error!("download of {} failed", &target_file);
        error!("error: {}", &e);
        return Err(e);
    }

    info!("download of {} succeeded!", &target_file);

    // On download success, sent a download result message out
    let file_size: u64;
    {
        file_size = ftb.read().await.get_file_size(&target_file).await?;
    }
    let download_result = Message::new(
        Header::new(
            user_identifier,
            serial_number,
            Timestamp::now()?,
        ),
        Payload::Download(
            DownloadMessage::Result {
                logical_file_path: target_file.clone(),
                file_size,
            }
        ),
    );

    ftb.read().await.socket.tx_message(download_result).await?;

    Ok(())
}

async fn leave(
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    let Some(serial_number) = NonZeroU64::new(1) else
    {
        return Err(anyhow!("failed to create serial number for leaving message"));
    };

    let ftb_handle = ftb.read().await;
    let message = Message::new(
        Header::new(
            ftb_handle.get_current_identifier(),
            SerialNumber(serial_number),
            Timestamp::now()?,
        ),
        Payload::Leaving,
    );

    Ok(ftb_handle.socket.tx_message(message).await?)
}

async fn read_line() -> anyhow::Result<String>
{
    let result = tokio::task::spawn_blocking(move || {
        let stdin = stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        Ok(line.trim().to_string())
    });

    result.await?
}

pub async fn run_ftb_ui(
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    let config = FiletreeBrowserConfiguration::load(FILETREE_BROWSER_CONFIGURATION_FILE)?;

    println!("port: {}, ipv6 addr: {}", config.m_port, config.m_addr6);

    info!("Starting FileTreeBrowser");
    info!("Configuration loaded: id={}, root_dir={}", config.uid, config.root_dir);
    debug!("Full config: {:?}", config);

    {
        ftb.read().await.print_list().await?;
    }

    loop
    {
        println!("\n[filename | '.' | '..' | ':services' | ':nodes' | ':search' | ':files' | ':download' | ':quit' | ':help'] ");
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

            // Sleep for a short time, then prompt the user for input again
            let sleep_time: u64;
            {
                sleep_time = ftb.read().await.config.sleep_time;
            }
            tokio::time::sleep(Duration::from_millis(sleep_time)).await;
        };

        debug!("User command: {}", user_cmd);

        match user_cmd.to_lowercase().as_str()
        {
            ":quit" => {
                info!("Quitting application");
                break;
            }

            ":help" => display_help().await?,

            ":services" => display_services(ftb.clone()).await,

            "." => ftb.read().await.print_list().await?,

            ".." => {
                let mut ftb_handle = ftb.write().await;
                if ftb_handle.this_dir == ftb_handle.root_path
                {
                    println!("At root : cannot move up.\n");
                    warn!("Attempt to move above root directory");
                }
                else if let Some(parent) = ftb_handle.get_parent()
                {
                    println!("<<< {}\n", ftb_handle.absolute_path_to_logical_path(&parent).await?);
                    info!("Moving up to: {}", ftb_handle.absolute_path_to_logical_path(&parent).await?);
                    ftb_handle.this_dir = parent;
                }
            }

            ":nodes" => nodes(ftb.clone()).await?,

            ":search" => search(ftb.clone()).await?,

            ":files" => files(ftb.clone()).await?,

            ":download" => {
                let result = download(ftb.clone()).await;
                if let Err(e) = result
                {
                    println!("{}", e);
                }
            },

            // The user has not entered any of the above commands, so assume they are requesting the
            // metadata for a specific file in the current working directory.
            _ => {
                let mut ftb_handle = ftb.write().await;
                if let Some(path) = ftb_handle.search_pwd_for_file(&user_cmd).await?
                {
                    let logical_path = ftb_handle.absolute_path_to_logical_path(&path).await?;
                    if path.is_file()
                    {
                        println!("file: {}", logical_path);
                        let file_size = ftb_handle.get_file_size(&logical_path).await?;
                        println!("size: {}", file_size);
                        info!(
                            "File accessed: {} (size: {})",
                            logical_path,
                            file_size,
                        );
                    }
                    else if path.is_dir()
                    {
                        println!(">>> {}", logical_path);
                        info!("Entering directory: {}", logical_path);
                        ftb_handle.this_dir = path;
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

    // Send a leaving message to indicate that this node has left the group.
    leave(ftb).await
}
