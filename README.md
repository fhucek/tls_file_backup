# TLS file backup
A TLS client/server that can be used to backup files and directories.<br>
- The Client takes a file or directory, archives it, compresses it, encrypts it over TLS, and sends it to a server.
    Much care is taken to ensure this is all done in-memory, meaning there is no need for a client to have extra drive space to create an archive to send the server.
    This makes this application *extremely* useful in backing up *very* large directories, such as `/home`.

- The Server receives these client requests and writes the data to a .tar.gz in a preconfigured backup directory.


## In production, get certs from valid CA
- Use certbot (or any other preferred method) to get yourself a valid cert/key for TLS
- `certbot certonly --standalone -d <your_domain>`
## Generate certs for development:
USE [mkcert](https://github.com/FiloSottile/mkcert) utility to generate certs and add ca root to trusted roots of system.
In public environment, use certificates from letsencrypt on server.


# After you're sure you have working certs;
# Running the .exe's (see releases)
- Releases are available for Linux x86_64, Linux aarch64 (arm64), Mac OSX Ventura
- Note: Client only works if the server is using a valid cert signed by a CA like LetsEncrypt. Client uses default system root store.
- Note: Server supplies valid cert/key pair. Get one from certbot.
- Note: Server and client both use port 4545 (make sure the server:4545 is accessible for the client).
        You can set default ports with command line option on either exe `--port <portno>`
- `server --cert "<absolute_path_to_cert_file>" --key "<absolute_path_to_key_file>" --backup-dir "<dir_to_put_archives>"`
- `client --host "<hostname_of_server_on_cert>" --file "<absolute_path_of_file_or_directory_to_send>"`


## Build
Tested on RPI, Mac OS, and Linux x86_64
`cargo build` `cargo build --release` for release builds
## Running the exes (devel - non release)
`cargo run --bin server -- --ip "0.0.0.0" --port 4545 --cert "<absolute_path_to_cert_file>" --key "<absolute_path_to_key_file>" --backup-dir <dir_to_put_archives>`

`cargo run --bin client -- --host "<server_ip_hostname_certdomain>" --port 4545 --file <absolute_path_of_file_to_send>`

## PLANS
Client needs to be able to request a file from the server. this shit is so fast.

## Personal notes
### How certs and client-validation of certs works
certificate authority (CA) signs a certificate (Csa) with its own CRT (CaCrt), producing End entity cert Ce.
Server hosting the TLS app (Sa) gets Csa & its own private key Ksa signed by CA and receives Ce.
Sa uses Ce and Ksa as its TLS encryption credentials.

Clients add CaCrt to their trusted certificates, trusting anything signed by CA.
Clients can now trust any Ce because it was signed by their already known and trusted CA bc of CaCrt.
Now, when a client initiates a connection to Sa, and Sa presents Ce, client trusts Sa because Ce was signed by CA



### Old 
### 

run server with env variable `RUST_LOG=trace cargo run --bin server` to view all logging

`openssl req -new -newkey rsa:4096 -x509 -sha256 -days 365 -nodes -out cert.crt -keyout priv.key`
put priv.key and cert.crt in a directory at the top level of this repo named certs/



openssl ecparam -genkey -name prime256v1 -noout -out server-private-key.pem

openssl ec -in server-private-key.pem -pubout -out server-public-key.pem

openssl req -new -x509 -sha256 -key server-private-key.pem -subj "/CN=duckduckgo.com" -out server-certificate.pem

Create garbage test file: `dd if=/dev/urandom of=random.img count=1024 bs=1M` or `fallocate -l 1G example.file`

## status
[x] TLS connection between client and server (w/ trusted CA on client explained above)

[x] Streaming messages between client and server

[X] Cleanup and abstract

[ ] Unit test `file_backup_service::connection::Connection` struct

[X] Client: Compress TLS stream buffer in memory (flate2 gzencoder) instead of compressing entire file before sending it<br>
        - Either way, if directory, you'd have to archive prior to streaming it anyway...

[ ] SHA file checksum

[X] tar.gz client input if directory

[X] Logging
    - timing of file transfers

[ ] Client "basic auth"
    - b/c this is over TLS, it's safe for the client to send a password over the network for the server to verify

[X] Errors

[X] Command line args
    - client input file


