//! Simple IPv6 link-local client
//! (based on Saleem's Java version)

use log::info;
use anyhow::anyhow;
use std::{
    io::{self},
    net::{SocketAddr},
    str::FromStr
};
use tokio::{
    fs::File,
    io::{
        AsyncReadExt,
        AsyncWriteExt,
        BufReader,
        BufWriter,
    },
    net::TcpStream
};
use std::path::PathBuf;
use crate::constants::BUFFER_SIZE;

/// The client is the node which contains a file which has been requested for download by another
/// node. The client will send the data to the server.
pub async fn run_client(
    server_addr: SocketAddr,
    requested_file: PathBuf,
) -> anyhow::Result<()>
{
    // We assume this requested file is an absolute path with "root_dir" as a prefix.
    // This is specified in the FileTreeBrowser instance.
    let target_file = File::open(requested_file).await?;

    println!("connecting to {}...", server_addr);

    let stream = TcpStream::connect(server_addr).await?;
    let mut total_bytes_sent: usize = 0;

    println!("connected! sending hello...\n");

    let mut file_reader = BufReader::new(target_file);
    let mut writer = BufWriter::new(stream);

    // read response
    loop
    {
        let mut buffer = [0u8; BUFFER_SIZE];
        file_reader.read(&mut buffer).await?;

        let result = writer.write(&mut buffer).await;
        writer.flush().await?;

        match result
        {
            Ok(n) if n > 0 => {
                println!("received ({} bytes):\n{}", n, String::from_utf8_lossy(&buffer[..n]));
                total_bytes_sent += n;
                info!("total bytes sent: {}", total_bytes_sent);
            }
            Ok(_) => {
                println!("connection closed");
                return Ok(())
            },
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                println!("interrupted, retrying")
            }
            Err(e) => {
                eprintln!("read error: {}", e);
                return Err(anyhow!(e));
            },
        }
    }

    Ok(())
}
