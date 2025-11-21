//! Simple IPv6 link-local client
//! (based on Saleem's Java version)

use log::info;
use std::{
    net::{SocketAddr},
};
use tokio::{
    fs::File,
    io::{
        AsyncWriteExt,
        BufWriter,
    },
    net::TcpStream
};
use std::path::PathBuf;

/// The client is the node which contains a file which has been requested for download by another
/// node. The client will send the data to the server.
pub async fn run_client(
    server_addr: SocketAddr,
    requested_file: PathBuf,
) -> anyhow::Result<()>
{
    // We assume this requested file is an absolute path with "root_dir" as a prefix.
    // This is specified in the FileTreeBrowser instance.
    let target_file = File::open(&requested_file).await?;

    info!("connecting to {}...", server_addr);

    let mut contents = tokio::fs::read(&requested_file).await?;

    info!("contents size: {}", contents.len());
    let stream = TcpStream::connect(server_addr).await?;

    info!("starting file tranfer");

    // Write the file contents to the TCP stream
    let mut writer = BufWriter::new(stream);
    let result = writer.write(&mut contents).await?;
    writer.flush().await?;
    info!("bytes sent: {:#?}", &result);
    tokio::io::stdout().flush().await?;

    Ok(())
}
