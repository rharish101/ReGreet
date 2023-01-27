//! Client that communicates with greetd
use std::env;
use std::io::Result as IOResult;
use std::os::unix::net::UnixStream;

use greetd_ipc::{
    codec::{Error as GreetdError, SyncCodec},
    Request, Response,
};
use tracing::info;

/// Environment variable containing the path to the greetd socket
const GREETD_SOCK_ENV_VAR: &str = "GREETD_SOCK";

pub type GreetdResult = Result<Response, GreetdError>;

/// The authentication status of the current greetd session
#[derive(Clone)]
pub enum AuthStatus {
    NotStarted,
    InProgress,
    Done,
}

/// Client that uses UNIX sockets to communicate with greetd
pub struct GreetdClient {
    /// Socket to communicate with greetd
    socket: UnixStream,
    /// Current authentication status
    auth_status: AuthStatus,
}

impl GreetdClient {
    /// Initialize the socket to communicate with greetd.
    pub fn new() -> IOResult<Self> {
        let sock_path = env::var(GREETD_SOCK_ENV_VAR).unwrap_or_else(|_| {
            panic!("Missing environment variable '{GREETD_SOCK_ENV_VAR}'. Is greetd running?",)
        });
        let socket = UnixStream::connect(sock_path)?;
        Ok(Self {
            socket,
            auth_status: AuthStatus::NotStarted,
        })
    }

    /// Initialize a greetd session.
    pub fn create_session(&mut self, username: &str) -> GreetdResult {
        info!("Creating session for username: {username}");
        let msg = Request::CreateSession {
            username: username.to_string(),
        };
        msg.write_to(&mut self.socket)?;

        let resp = Response::read_from(&mut self.socket)?;
        match resp {
            Response::Success => {
                self.auth_status = AuthStatus::Done;
            }
            Response::AuthMessage { .. } => {
                self.auth_status = AuthStatus::InProgress;
            }
            Response::Error { .. } => {
                self.auth_status = AuthStatus::NotStarted;
            }
        };
        Ok(resp)
    }

    /// Send password to a greetd session.
    pub fn send_password(&mut self, password: Option<String>) -> GreetdResult {
        info!("Sending password to greetd");
        let msg = Request::PostAuthMessageResponse { response: password };
        msg.write_to(&mut self.socket)?;

        let resp = Response::read_from(&mut self.socket)?;
        match resp {
            Response::Success => {
                self.auth_status = AuthStatus::Done;
            }
            Response::AuthMessage { .. } => {
                self.auth_status = AuthStatus::InProgress;
                unimplemented!("greetd responded with auth request after sending password.");
            }
            Response::Error { .. } => {
                self.auth_status = AuthStatus::InProgress;
            }
        };
        Ok(resp)
    }

    /// Schedule starting a greetd session.
    ///
    /// On success, the session will start when this greeter terminates.
    pub fn start_session(&mut self, command: Vec<String>) -> GreetdResult {
        info!("Starting greetd session with command: {command:?}");
        let msg = Request::StartSession {
            cmd: command,
            env: Vec::new(),
        };
        msg.write_to(&mut self.socket)?;

        let resp = Response::read_from(&mut self.socket)?;
        if let Response::AuthMessage { .. } = resp {
            unimplemented!("greetd responded with auth request after requesting session start.");
        }
        Ok(resp)
    }

    /// Cancel an initialized greetd session.
    pub fn cancel_session(&mut self) -> GreetdResult {
        info!("Cancelling greetd session");
        self.auth_status = AuthStatus::NotStarted;

        let msg = Request::CancelSession;
        msg.write_to(&mut self.socket)?;

        let resp = Response::read_from(&mut self.socket)?;
        if let Response::AuthMessage { .. } = resp {
            unimplemented!(
                "greetd responded with auth request after requesting session cancellation."
            );
        }
        Ok(resp)
    }

    pub fn get_auth_status(&self) -> &AuthStatus {
        &self.auth_status
    }
}

impl Drop for GreetdClient {
    fn drop(&mut self) {
        // Cancel any created session, just to be safe.
        self.cancel_session()
            .expect("Couldn't cancel session on exit.");
    }
}
