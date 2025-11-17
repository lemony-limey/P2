//! Simple IPv6 link-local client
//! (based on Saleem's Java version)

use anyhow::anyhow;
use std::io::BufWriter;
use std::{
    env,
    ffi::CString,
    io::{self, Read, Write},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpStream},
    str::FromStr
};
use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroU16;
use std::path::PathBuf;
use crate::constants::BUFFER_SIZE;

/// The client is the node which contains a file which has been requested for download by another
/// node. The client will send the data to the server.
pub fn run_client(
    server_addr: SocketAddr,
    requested_file: PathBuf,
) -> anyhow::Result<()>
{
    // We assume this requested file is an absolute path with "root_dir" as a prefix.
    // This is specified in the FileTreeBrowser instance.
    let target_file = File::open(requested_file)?;

    println!("connecting to {}...", server_addr);

    let stream = TcpStream::connect(server_addr)?;
    let mut total_bytes_sent: usize = 0;
    let mut total_bytes_received: usize = 0;

    println!("connected! sending hello...\n");

    let mut file_reader = BufReader::new(target_file);
    let mut writer = BufWriter::new(stream);

    // read response
    loop
    {
        let mut buffer = [0u8; BUFFER_SIZE];
        file_reader.read(&mut buffer)?;

        // while let Ok(bytes_written) = writer.write()
        // {
        //     if bytes_written > 0
        //     {
        //         println!("written ({} bytes):\n{}", bytes_written, String::from_utf8_lossy(&file_reader.buffer()[total_bytes_received..total_bytes_received + bytes_written]));
        //         total_bytes_received += bytes_written;
        //     }
        //     else if bytes_written == 0
        //     {
        //         println!("connection closed by peer with no data");
        //         return Ok(())
        //     }
        // }
        match writer.write(&mut buffer)
        {
            Ok(n) if n > 0 => {
                println!("received ({} bytes):\n{}", n, String::from_utf8_lossy(&buffer[..n]));
                total_bytes_received += n;
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
