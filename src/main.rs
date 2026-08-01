use std::time::Duration;
use std::{net::TcpStream, sync::Arc};
use cargo::sources::git::fetch::Error;
use rustls::{Stream, pki_types::ServerName};
use tokio::time::sleep;

use crate::init::chk_if_managed;
use crate::logging::*;
use crate::net::{Poll, PollResponse, Server};

pub mod init;
pub mod net;
pub mod logging;
pub mod util;
pub mod state;

type PollResponseResult = Result<PollResponse, std::io::Error>;

// TODO: devise certain methods annd actions which will trigger a pull 

#[tokio::main]
async fn main() {
    let mut server = match Server::<TcpStream>::new("localhost", 8080, None) {
         Ok(s)  => s,
         Err(e) => {
             log!(LogLevel::Error, "Error connecting to server - {}", e.to_string(); 101);
             return;
         } 
    };
    
    let is_managed = match chk_if_managed().await {
        Ok(b) => b,
        Err(e) => {
            log!(LogLevel::Error, "Failure to check if machine is manged due to error - {}", e.to_string(); 250);
            return; // this return wont ever be reached but is required to make the compiler happy  
        }
    };

    if !is_managed {
        log!(LogLevel::Error, "{}", "This machine is not managed. Use navix-enroll to enroll the machine."; 251);
        return;
    }


    loop {
        let response_result: PollResponseResult = server.request::<Poll, PollResponse>(Poll::new());
        if response_result.is_err() {
            log!(LogLevel::Warn, "Unable to poll the managment server - {}", response_result.unwrap_err().to_string());
            sleep(Duration::from_hours(1)).await;
            continue;
        }

        match handle_poll_response(response_result.unwrap()) {
            Ok(_) => (),
            Err(e) => ()
        };

        sleep(Duration::from_hours(1)).await;
    }
}




fn handle_poll_response(response: PollResponse) -> Result<(), Box<dyn std::error::Error>> {
    
    if response.wipe {
        // TODO: implement wipe functionality 
    }

    if response.rebuild {
        // TODO: implement rebuild functionality
    }

    if response.script {
        // TODO: implement script runner 
    }

    if response.rotate_root_pwd {
        // TODO: implement root password rotation 
    }

    if response.rotate_luks {
        // TODO: implement luks & the rotation of the keys
    }

    if response.malware_scan {
        // TODO: implement a generic scanner depending upon what administrators install 
    }

    return Ok(());
}
