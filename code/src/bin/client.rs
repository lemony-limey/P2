use std::net::SocketAddr;
use std::path::PathBuf;
use std::num::NonZeroU16;
use anyhow;
use std::env;
use tokio::io::AsyncWriteExt;
use code::simple_tcp::parse_ipv6_with_scope;
use code::simple_tcp::client::run_client;

/// This is only used if the client is run from the command line, not when initiated by a UDP
/// multicast request.
#[tokio::main]
pub async fn main() -> anyhow::Result<()>
{
    let mut args = env::args().skip(1);
    let server = match args.next()
    {
        Some(s) => s,
        None => {
            eprintln!("usage: client <server-ip-or-host> <port> <file_name>");
            std::process::exit(2);
        }
    };

    let port: NonZeroU16 = match args.next()
    {
        Some(p) => p.parse().expect("port must be a number"),
        None => {
            eprintln!("usage: client <server-ip-or-host> <port> <file_name>");
            std::process::exit(2);
        }
    };

    let file_name = match args.next()
    {
        Some(s) => s,
        None => {
            eprintln!("usage: client <server-ip-or-host> <port> <file_name>");
            std::process::exit(2);
        }
    };

    // build SocketAddr taking into account IPv6 link-local scope ids
    let addr = match parse_ipv6_with_scope(&server, port)
    {
        Ok(SocketAddr::V6(a)) => a,
        _ => {
            eprintln!("Could not resolve address");
            std::process::exit(1);
        },
    };

    let result = run_client(addr, PathBuf::from(file_name)).await;
    println!("result: {:#?}", result);
    tokio::io::stdout().flush().await?;

    Ok(())
}
