use clap::Parser;
use log::{error, info};

use std::io;

use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::TlsConnector;

use file_backup_service::common;
use file_backup_service::connection;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ClientArgs {
    #[arg(short, long)]
    host: String,
    #[arg(short, long, default_value_t = 4545)]
    port: i32,
    #[arg(short, long)]
    file: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    common::setup_logger();
    let args = ClientArgs::parse();
    let host = common::make_address_str(&args.host, &args.port);

    let (absolute_path_to_archive_and_send, archivename_to_tell_server) =
        common::get_fileinfo_to_send(&args.file)?;

    //// TLS Setup ////
    info!("Connecting to {}", host);
    let addr = host
        .to_string()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;

    let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
    let certlist = tokio::task::spawn_blocking(|| {
        rustls_native_certs::load_native_certs().expect("Could not load platform certs")
    })
    .await?;
    for cert in certlist {
        root_cert_store.add(cert).unwrap();
    }
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()); // maybe make this async

    let ip_addr = ServerName::try_from(args.host).unwrap();
    let config = tokio_rustls::rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let tls_connector = TlsConnector::from(Arc::new(config));

    let sock_stream = TcpStream::connect(&addr).await?;
    let tls_stream = tls_connector.connect(ip_addr, sock_stream).await?;
    let mut conn = connection::Connection::new(tls_stream);
    info!("TLS connection established with {}", host);
    //// TLS Setup ////

    // sequential message passing with server
    let filename_message_to_send = format!("filename:{}:filename", archivename_to_tell_server);
    conn.write_message_from_string(filename_message_to_send)
        .await?;

    let server_response = conn.read_into_string().await?;
    if server_response != "OK" {
        let msg = "Server sent a bad response to our file request. Aborting...".to_string();
        error!("{}", msg);
        panic!("{}", msg);
    }

    info!(
        "Received ok from server. Sending {}",
        absolute_path_to_archive_and_send
    );

    conn.compress_and_send(absolute_path_to_archive_and_send)
        .await?;

    info!("Client done. Exiting.");
    Ok(())
}

// use this and just distribute client with .crt?
#[cfg(debug_assertions)]
use rustls::RootCertStore;
#[cfg(debug_assertions)]
fn _add_cafile_to_root_store(roots: &mut RootCertStore, certfile: String) -> Result<(), io::Error> {
    use std::fs::File;
    use std::io::BufReader;
    // USE this to include CA crt file with which to accept anyone's cert the CA has signed
    // very useful to distribute client with CA cert
    println!("OPENING CERT FILE {}", certfile);
    let mut pem = BufReader::new(File::open(certfile)?);
    for cert in rustls_pemfile::certs(&mut pem) {
        let cert = match cert {
            Ok(cert) => {
                println!("Got a cert");
                cert
            }
            Err(_) => {
                println!("Err occurred ");
                break;
            }
        };
        roots.add(cert).unwrap();
    }
    Ok(())
}
