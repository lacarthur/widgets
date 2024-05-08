pub mod subscription;
pub mod ui;

use serde::Deserialize;
use std::{collections::HashMap, env::VarError, fmt::Display};

// I am not dealing with dynamic workspaces.
pub const NUM_WORKSPACES: usize = 10;
pub const HYPRLAND_INSTANCE_SIG_VAR: &str = "HYPRLAND_INSTANCE_SIGNATURE";

#[derive(Deserialize, Debug)]
struct WorkspaceDeserialized {
    id: usize,
}

#[derive(Deserialize, Debug)]
struct HyprlandClientDeserialized {
    address: String,
    workspace: WorkspaceDeserialized,
}

#[derive(Debug)]
pub enum HyprlandCommunicationError {
    IoError { command: String, error: std::io::Error },
    DeserializationError { command: String, raw: String, error: serde_json::Error },
    HexadecimalMissingPrefix { command: String, address: String },
    WindowAddressParsingError { command: String, address: String, error: std::num::ParseIntError },
    HyprctlFailure { command: String, exit_status: std::process::ExitStatus },
    SocketConnectionError { socket_path: String, error: std::io::Error },
    EventParsingError { event: String },
    EventArgsParsingError { event: String, args: String },
    RequestInexistantWindow { requested_address: u64, addresses_in_memory: Vec<u64> },
    EnvError { var: String, error: VarError },
}

impl Display for HyprlandCommunicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError { command, error } => {
                writeln!(f, "IO Error when communicating with Hyprland through hyprctl.")?;
                writeln!(f, "Ran command '{}'", command)?;
                write!(f, "Got error '{}'", error)
            }
            Self::DeserializationError { command, raw, error } => {
                writeln!(f, "Error while deserializing JSON data received from hyprctl")?;
                writeln!(f, "Ran command '{}'", command)?;
                writeln!(f, "Got answer : ")?;
                writeln!(f, "{}", raw)?;
                write!(f, "And error : '{}'", error)
            }
            Self::HexadecimalMissingPrefix { command, address } => {
                writeln!(f, "Expected the window address to start with '0x'. This may have changed.")?;
                writeln!(f, "Ran command '{}", command)?;
                writeln!(f, "Extracted the following window address : {}, which does not start with 0x, contrary to what was expected.", address)
            }
            Self::WindowAddressParsingError { command, address, error } => {
                writeln!(f, "Encountered error while parsing window address.")?;
                writeln!(f, "While running the command '{}'", command)?;
                writeln!(f, "Encountered the following address: {}", address)?;
                writeln!(f, "and produced this error while attempting to parse it: {}", error)
            }
            Self::HyprctlFailure { command, exit_status } => {
                writeln!(f, "hyprctl failed.")?;
                writeln!(f, "Ran command : '{}'", command)?;
                writeln!(f, "Received ExitStatus : {}", exit_status)
            }
            Self::SocketConnectionError { socket_path, error } => {
                writeln!(f, "Error while attempting to connect to socket at address {}", socket_path)?;
                write!(f, "Received error '{}'", error)
            }
            Self::EventParsingError { event } => {
                writeln!(f, "Error while parsing the event got from hyprctl.")?;
                writeln!(f, "Event received : {}", event)?;
                write!(f, "Expected format : 'event_name>>args'")
            }
            Self::EventArgsParsingError { event, args } => {
                writeln!(f, "Error while parsing args from hyprctl.")?;
                writeln!(f, "Received this event : '{}'", event)?;
                writeln!(f, "Extracted these args : '{}'", args)?;
                writeln!(f, "Expected them to be separated by commas.")
            }
            Self::RequestInexistantWindow { requested_address, addresses_in_memory } => {
                writeln!(f, "Requesting inexistant window : {}", requested_address)?;
                write!(f, "The following addresses were in memory : ")?;
                for address in addresses_in_memory {
                    write!(f, "{} ", address)?;
                }
                Ok(())
            }
            Self::EnvError { var, error } => {
                writeln!(f, "Error accessing environment variable : {}", var)?;
                write!(f, "Got error {}", error)
            }
        }
    }
}

pub fn get_windows() -> Result<(HashMap<u64, usize>, [u32; NUM_WORKSPACES]), HyprlandCommunicationError> {
    let command = "hyprctl clients -j";
    let query_output = std::process::Command::new("hyprctl")
        .arg("clients")
        .arg("-j")
        .output()
        .map_err(|error| {
            HyprlandCommunicationError::IoError { 
                command: command.into(),
                error
            }
        })?;

    let data = std::str::from_utf8(&query_output.stdout).unwrap();

    let hyprland_clients_list: Vec<HyprlandClientDeserialized> = serde_json::from_str(data)
        .map_err(|error| HyprlandCommunicationError::DeserializationError { 
            command: command.into(), 
            raw: data.into(), 
            error,
        })?;

    let mut windows = HashMap::new();
    let mut count_windows = [0; NUM_WORKSPACES];

    for client in hyprland_clients_list {
        let id = client.workspace.id - 1;
        count_windows[id] += 1;
        let address = client.address.strip_prefix("0x")
            .ok_or(HyprlandCommunicationError::HexadecimalMissingPrefix { 
                command: command.into(), 
                address: client.address.clone().into() 
            })?;
        let address = u64::from_str_radix(address, 16)
            .map_err(|error| HyprlandCommunicationError::WindowAddressParsingError { 
                command: command.into(), 
                address: address.into(), 
                error  
        })?;
        windows.insert(address, id);
    }
    Ok((windows, count_windows))
}

pub fn get_active_workspace() -> Result<usize, HyprlandCommunicationError> {
    let command = "hyprctl activeworkspace -j";
    let query_output = std::process::Command::new("hyprctl")
        .arg("activeworkspace")
        .arg("-j")
        .output()
        .map_err(|error| HyprlandCommunicationError::IoError { 
            command: command.into(), 
            error 
        })?;

    let data = std::str::from_utf8(&query_output.stdout).unwrap();

    let active_workspace: WorkspaceDeserialized = serde_json::from_str(data)
        .map_err(|error| HyprlandCommunicationError::DeserializationError { command: command.into(), raw: data.into(), error })?;

    Ok(active_workspace.id - 1)
}

pub fn switch_to_workspace(new_workspace_id: usize) -> Result<(), HyprlandCommunicationError> {
    let command = format!("hyprctl dispatch workspace {}", new_workspace_id + 1);
    let status = std::process::Command::new("hyprctl")
        .arg("dispatch")
        .arg(format!("workspace {}", new_workspace_id + 1))
        .status()
        .map_err(|error| HyprlandCommunicationError::IoError { command: command.clone(), error })?;
    match status.success() {
        true => Ok(()),
        false => Err(HyprlandCommunicationError::HyprctlFailure { command, exit_status: status }),
    }
}
