//! Simple non-blocking TCP server using IPv6 link-local addresses
//! (based on Saleem's Java version)

use tokio::io::{BufReader, BufWriter};
use tokio::fs::File;
use crate::{
    constants::{BUFFER_SIZE},
    simple_tcp::{anyhow, self},
};
use std::{
    net::{SocketAddr, IpAddr, Ipv6Addr},
};
use std::path::PathBuf;
use log::{info, error};
use tokio::fs;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The server is the node requesting the download of the file from another node.
/// It will receive the file from the client.
pub async fn run_server(
    file_requested_from_client: PathBuf,
) -> anyhow::Result<u64>
{
    // info!("Starting TCP server");
    println!("Starting TCP server");
    tokio::io::stdout().flush().await?;

    let Some(port) = simple_tcp::get_uid_port() else { return Err(anyhow!("oh no")) };
    // info!("Using port: {}", port);
    println!("Using port: {}", port);
    tokio::io::stdout().flush().await?;

    // For link-local binding, we need to bind to the unspecified address
    // The system will handle the interface selection
    let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port.get());
    
    // info!("Binding to: {}", bind_addr);
    println!("Binding to: {}", bind_addr);
    tokio::io::stdout().flush().await?;
    let listener = TcpListener::bind(bind_addr).await?;

    // Log the actual link-local address for reference
    if let Ok((ipv6_addr, interface)) = simple_tcp::get_link_local_ipv6() 
    {
        // info!("Link-local IPv6 available: {} on interface {}", ipv6_addr, interface);
        // info!("Clients should connect to: [{}%{}]:{}", ipv6_addr, interface, port);
        println!("Link-local IPv6 available: {} on interface {}", ipv6_addr, interface);
        println!("Clients should connect to: [{}%{}]:{}", ipv6_addr, interface, port);
        tokio::io::stdout().flush().await?;
    }
    
    // info!("Waiting for connection...");
    println!("Waiting for connection...");
    tokio::io::stdout().flush().await?;

    let new_file = File::create_new(&file_requested_from_client).await?;
    let mut file_writer = BufWriter::new(new_file);
    let mut total_bytes_received: u64 = 0;

    // Wait for a connection
    let stream = loop
    {
        match listener.accept().await
        {
            Ok((stream, addr)) => {
                // info!("New connection from: {}", addr);
                println!("New connection from: {}", addr);
                tokio::io::stdout().flush().await?;
                break stream;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No incoming connection, keep trying
            }
            Err(e) => {
                // error!("Error accepting connection: {}", e);
                println!("Error accepting connection: {}", e);
                tokio::io::stdout().flush().await?;
                return Err(e.into());
            }
        }
    };

    // No longer need to accept connections
    drop(listener);
    // info!("listener dropped");
    println!("listener dropped");
    tokio::io::stdout().flush().await?;

    // Read the contents of the stream into the file.
    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    let bytes_received = reader.read_to_end(&mut buffer).await?;
    tokio::fs::write(file_requested_from_client, buffer).await?;

    // Echo server loop
    // loop
    // {
    //     let mut buffer = vec![0u8; BUFFER_SIZE];
    //     match reader.read(&mut buffer).await
    //     {
    //         Ok(0) => {
    //             // info!("<Connection closed by client>");
    //             println!("<Connection closed by client>");
    //             tokio::io::stdout().flush().await?;
    //             break;
    //         }
    //         Ok(bytes_read) => {
    //             let received = String::from_utf8_lossy(&buffer[..bytes_read]);
    //             total_bytes_received += bytes_read as u64;
    //             // info!("Received {} bytes --> {}; total bytes received: {}", bytes_read, received.trim(), total_bytes_received);
    //             println!("Received {} bytes --> {}; total bytes received: {}", bytes_read, received.trim(), total_bytes_received);
    //             tokio::io::stdout().flush().await?;
    //
    //             // Write to file
    //             file_writer.write(&mut buffer).await?;
    //             file_writer.flush().await?;
    //         }
    //         Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
    //                || e.kind() == std::io::ErrorKind::TimedOut => {
    //             // No data available, continue waiting
    //         }
    //         Err(e) => {
    //             // error!("Error reading from client: {}", e);
    //             // info!("<Closing connection>");
    //             println!("Error reading from client: {}", e);
    //             println!("<Closing connection>");
    //             tokio::io::stdout().flush().await?;
    //             return Err(anyhow!(e));
    //         }
    //     }
    // }

    // info!("<Closing connection>");
    println!("<Closing connection>");
    tokio::io::stdout().flush().await?;
    Ok(bytes_received as u64)
}
