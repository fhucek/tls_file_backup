use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::path::Path;
use file_backup_service::connection::ClientConnection;
use rustls_pemfile::ec_private_keys;
use rustls_pemfile::{certs, rsa_private_keys, pkcs8_private_keys};
use rustls::ServerConfig;
use tokio::io::AsyncReadExt;
use tokio::io::{copy, sink, split, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::pki_types::CertificateDer;
use tokio_rustls::rustls::pki_types::PrivateKeyDer;
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;

use file_backup_service::common;
use file_backup_service::connection;


const MY_ADDR: &str = "0.0.0.0:4545";
// const CERT: &str = "certs/new_localhost/server-certificate.pem";
// const KEY: &str = "certs/new_localhost/server-private-key.pem";
const CERT: &str = "/home/frank/certs/ripplein.space-dev.pem";
const KEY: &str = "/home/frank/certs/ripplein.space-dev-key.pem";

// curl --cacert certs/new/server-certificate.pem https://ripplein.space:4545/ --resolve ripplein.space:4545:127.0.0.1
fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_keys(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .next()
        .unwrap()
        .map(Into::into)
}

fn load_ec_keys(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    ec_private_keys(&mut BufReader::new(File::open(path)?))
        .next()
        .unwrap()
        .map(Into::into)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = MY_ADDR
        .to_string()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;

    let certs = load_certs(&Path::new(CERT))?;
    let key = load_keys(&Path::new(KEY))?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    let tlsacceptor =  TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let tlsacceptor_ptr = tlsacceptor.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(socket, peer_addr, tlsacceptor_ptr).await {
                eprintln!("{:?}", err);
            };
        });
    }

    Ok(())
}


async fn handle_client(sock: TcpStream, peer_addr: SocketAddr, tls_acceptor: TlsAcceptor) -> Result<(), Box<dyn Error>>{
        // handshake to establish encrypted connection
        let tls_stream = match tls_acceptor.accept(sock).await {
            Ok(tls_stream) => tls_stream,
            Err(_err) => { println!("ERROR HANDLING CLIENT {}", _err); return Err(Box::new(_err)); } // return early if accept fails, nothing to do
        };

        println!("Connected via TLS to {}", peer_addr);

        let mut server_conn = file_backup_service::connection::ServerConnection::new(tls_stream);

        let mut string = match server_conn.read_message_into_string().await {
            Ok(string) => string,
            Err(_) => {
                panic!("failed read to server")
            }
        };

        println!("Received this message from client: {}", string);
        string.push_str(" Addendum from server");

        match server_conn.write_message_from_string(string).await {
            Ok(_) => (),
            Err(_) => {
                panic!("failed read to server")
            }
        };

        server_conn.shutdown_tls_conn().await?;
        
        Ok(())
}
