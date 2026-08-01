use std::{io::{Error, ErrorKind, Read, Write}, net::TcpStream, str::from_utf8, sync::Arc};
use rustls::{ClientConnection, StreamOwned, pki_types::ServerName};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use crate::{state::*};
use crate::init::{HardwareFingerprint};


// TODO: Move to protobuf eventually, probably. kinda lazy, serde is ez & will work for now 
// Registration struct stuff ===

#[derive(Serialize)]
pub struct Registration {
     fingerprint: HardwareFingerprint,
     enrollment_token: String
}

impl Registration {
    pub fn new(token: String, fingerprint: String) -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            fingerprint: fingerprint,
            enrollment_token: token
        });
    }
}


// Poll struct stuff ===


#[derive(Deserialize, Serialize, Debug)]
pub struct PollResponse {
    pub wipe: bool,
    pub rebuild: bool,
    pub script: bool,
    pub reboot: bool,
    pub rotate_root_pwd: bool,
    pub rotate_luks: bool,
    pub malware_scan: bool 
}

impl PollResponse {
    pub fn new() -> Self {
        return Self {
            wipe: false,
            rebuild: false,
            script: false,
            reboot: false,
            rotate_root_pwd: false,
            rotate_luks: false,
            malware_scan: false
        }
    }   
}

#[derive(Serialize)]
pub struct Poll {
    is_rebuilding: bool,
    configuration: String,
    last_sync_timestamp: String 
}

impl Poll {
    pub fn new() -> Self {  
        return Self {
            is_rebuilding: false, // TODO,
            configuration: "default".to_string(), // TODO
            last_sync_timestamp: get_last_sync_timestamp().unwrap_or("failed".to_string()) 
        }
    }
}


// ===

fn get_root_store() -> rustls::RootCertStore {
    return rustls::RootCertStore::from_iter(
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
    );
}

fn default_tls_cfg() -> Arc<rustls::ClientConfig> {
    return Arc::new(rustls::ClientConfig::builder()
        .with_root_certificates(get_root_store())
        .with_no_client_auth());
}

fn default_state(hostname: impl AsRef<str>) -> Result<ClientConnection, Error> {
    let server = match ServerName::try_from(hostname.as_ref()) {
        Ok(s) => s,
        Err(_) => return Err(Error::new(ErrorKind::Other, "Invalid DNS name or IP address"))
    };

    return Ok(
        match ClientConnection::new(default_tls_cfg(), server.to_owned()) {
            Ok(c) => c,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string()))
        }
    );
}

// returns first instance found 
fn subslice_contains<T: PartialEq>(slice: &[T], target: &[T]) -> bool {
    for i in 0..slice.len() {
        if slice[i] == target[0] {
            for j in 1..target.len() {
                if slice[i + j] != target[j] { break; }
                if slice[i + j] == target[j] && j == target.len() - 1 { return true; }
            }
        }
    }
    return false;
} 


// trait for IO actions that need to be performed on the duplex 
pub trait IOStream: Read + Write {
    fn close(&mut self) -> Result<(), Error>;
    fn connect(addr: impl AsRef<str>) -> Result<Self, Error> where Self: Sized;
}


impl IOStream for std::net::TcpStream {
    fn close(&mut self) -> Result<(), Error> {
        match self.shutdown(std::net::Shutdown::Both) {
            Ok(_) => return Ok(()),
            Err(e) => match e.kind() {
                ErrorKind::Interrupted => return Ok(()),
                _ => return Err(e)
            }

        }
    }    

   fn connect(addr: impl AsRef<str>) -> Result<Self, Error> {
       return TcpStream::connect(addr.as_ref());
   }
}



struct TLSStream<S: IOStream>(StreamOwned<ClientConnection, S>);

impl<S: IOStream> Read for TLSStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        return self.0.read(buf);
    }
}

impl<S: IOStream> Write for TLSStream<S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        return self.0.write(buf);
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return self.0.flush();
    }
} 

impl<S: IOStream> TLSStream<S> {
    fn read_until_delimiter(&mut self, buf: &mut [u8], delimiter: &[u8]) -> Result<usize, Error> {
        let mut loops = 0;
        let mut len = 0;
        loop {
            loops = loops + 1;
            if loops > 100 {
                return Err(Error::new(ErrorKind::TimedOut, "Abnormal amount of reads from the server for one message"));
            }

            match self.read(buf) {
                Ok(l) => len = len + l,
                Err(e) => {
                    match e.kind() {
                        ErrorKind::Interrupted => continue,
                        _ => return Err(e)
                    }
                }
            };

            if subslice_contains(&buf, &delimiter) { return Ok(len); }
        }
    }

}


pub struct Server<S: IOStream> {
    pub hostname: String,
    pub port: u32,
    tls_state: Option<ClientConnection>,
    tls_stream: Option<TLSStream<S>>
}

impl<S: IOStream> Server<S> {
    pub fn new(hostname: impl AsRef<str> + Clone, port: u32, mut state: Option<ClientConnection>) -> Result<Self, Error> {
        if state.is_none() {
            state = Some(default_state(hostname.clone())?);
        }
        
        return Ok(Self {
            hostname: hostname.as_ref().to_string(),
            port: port,
            tls_state: state,
            tls_stream: None
        });
    }


    fn addr(&self) -> String {
        return format!("{}:{}", self.hostname, self.port);
    }

    fn init_connection(&mut self, stream: S) -> () {
        self.tls_stream = Some(TLSStream(StreamOwned::new(self.tls_state.take().unwrap(), stream)));
    }
    

    pub fn request<T: Serialize, RT: DeserializeOwned>(&mut self, request: T) -> Result<RT, Error> {
        if self.tls_stream.is_none() {
            let stream = S::connect(self.addr())?;
            self.init_connection(stream);  
        }

        let json = serde_json::to_vec(&request)?;
        let mut response_buffer = [0u8; 4096];
        let delimiter = b"\r\n\r\n";
        let mut tls_stream = self.tls_stream.take().unwrap();

        tls_stream.write_all(&json)?;
        tls_stream.write_all(delimiter)?;
        tls_stream.flush()?;
        let recv_len = tls_stream.read_until_delimiter(&mut response_buffer, delimiter)?;
        return Ok(serde_json::from_slice::<RT>(&response_buffer[..recv_len - 4])?);
    }

}

// a bunch of scaffolding for the sake of testing but its kinda whatever
// ill probably just end up reusing this in another crate 

      


#[cfg(test)]
mod tests {
use rustls_pki_types::CertificateDer;
use rustls::ServerConnection;
use std::os::unix::net::UnixStream;
use super::*;

#[cfg(test)] 
impl IOStream for UnixStream {
    fn connect(addr: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
        return Ok(UnixStream::connect(addr.as_ref()).unwrap());
    }

    fn close(&mut self) -> Result<(), Error> {
        self.shutdown(std::net::Shutdown::Both).unwrap();
        return Ok(());
    }
}


#[cfg(test)]
fn test_server_tls_cfg() -> (rustls::ServerConfig, CertificateDer<'static>) {
    let rcgen::CertifiedKey { cert, signing_key } = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    return (rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert.der().clone()], signing_key.serialize_der().try_into().unwrap()).unwrap(), cert.der().clone());
}

#[cfg(test)]
fn test_client_tls_cfg(server_cert_der: rustls_pki_types::CertificateDer) -> rustls::ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(server_cert_der).unwrap();

    rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth()
}

#[test]
fn subslice_contains_test() {
    let delimiter = b"\r\n\r\n".to_vec();
    let example = PollResponse::new(); 
    let mut data = serde_json::to_vec(&example).unwrap();
    data.append(&mut delimiter.clone());
    let data2 = b"foo bar\r\n\rfoo bar";
    assert!(subslice_contains(&data,&delimiter));
    assert!(!subslice_contains(data2,&delimiter))
}


#[test]
fn read_across_multiple_writes() {
    let (server_socket, client_socket) = UnixStream::pair().unwrap();
    let (server_config, server_cert) = test_server_tls_cfg();
    let client_config                = test_client_tls_cfg(server_cert);

    let state = ClientConnection::new(Arc::new(client_config), ServerName::try_from("localhost").unwrap()).unwrap();
    let server_state = ServerConnection::new(Arc::new(server_config)).unwrap();
    
    let task = std::thread::spawn(move || {
        let mut tls_stream = rustls::StreamOwned::new(server_state, server_socket);
        let mut buf = [0u8; 4096];
        tls_stream.read(&mut buf).unwrap();
        let poll_resp = PollResponse::new();
        let json = serde_json::to_vec(&poll_resp).unwrap();
        tls_stream.write(&json).unwrap();
        tls_stream.write(b"\r\n\r\n").unwrap();
        tls_stream.flush().unwrap();
    });

    let task2 = std::thread::spawn(move || {    
        let mut server = Server::<UnixStream>::new("localhost", 0, Some(state)).unwrap();
        server.init_connection(client_socket);        
        let _: PollResponse = server.request(Poll::new()).unwrap();
    });

    task.join().unwrap();
    task2.join().unwrap();
    
}
}





