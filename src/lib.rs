use prettytable::{cell, format, row, table, Cell, Table};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct APIErrorResponse {
    errcode: String,
    error: String,
    soft_logout: Option<bool>,
}

#[derive(Debug)]
pub struct APIErrorMessage {
    api_error_response: APIErrorResponse,
    status_code: u16,
}

impl APIErrorMessage {
    fn new(api_error_response: APIErrorResponse, status_code: u16) -> APIErrorMessage {
        APIErrorMessage {
            api_error_response,
            status_code,
        }
    }
}

#[derive(Debug)]
pub enum MatrixAPIError {
    ServerNotDefined(String),
    ConfigFileError(String),
    AccessTokenError(String),
    ReqwestError(String),
    APIRequestError(APIErrorMessage),
}

impl std::error::Error for MatrixAPIError {}

impl fmt::Display for MatrixAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatrixAPIError::ServerNotDefined(s) => {
                write!(f, "The given server {} is not defined", s)
            }
            MatrixAPIError::ConfigFileError(s) => {
                write!(f, "There was an error with the config file: {}", s)
            }
            MatrixAPIError::AccessTokenError(s) => {
                write!(f, "There was an error reading the access token: {}", s)
            }
            MatrixAPIError::ReqwestError(s) => {
                write!(f, "There was an error during the HTTP request: {}", s)
            }
            MatrixAPIError::APIRequestError(e) => {
                write!(f, "There was an error running the API request.\n\terror:\t\"{}\"\n\terrcode\t\"{}\"\n\tstatus:\t{}", e.api_error_response.error, e.api_error_response.errcode, e.status_code)
            }
        }
    }
}

impl From<std::io::Error> for MatrixAPIError {
    fn from(error: std::io::Error) -> Self {
        MatrixAPIError::ConfigFileError(error.to_string())
    }
}

impl From<xdg::BaseDirectoriesError> for MatrixAPIError {
    fn from(error: xdg::BaseDirectoriesError) -> Self {
        MatrixAPIError::ConfigFileError(error.to_string())
    }
}

impl From<toml::de::Error> for MatrixAPIError {
    fn from(error: toml::de::Error) -> Self {
        MatrixAPIError::ConfigFileError(error.to_string())
    }
}

impl From<std::str::Utf8Error> for MatrixAPIError {
    fn from(error: std::str::Utf8Error) -> Self {
        MatrixAPIError::AccessTokenError(error.to_string())
    }
}

impl From<reqwest::Error> for MatrixAPIError {
    fn from(error: reqwest::Error) -> Self {
        MatrixAPIError::ReqwestError(error.to_string())
    }
}

#[derive(Deserialize, Debug)]
struct Config {
    default_server: String,
    server: HashMap<String, ServerConfig>,
}

#[derive(Deserialize, Debug, Clone)]
struct ServerConfig {
    server_name: String,
    server_url: String,
    pass_access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserList {
    total: u32,
    users: Vec<User>,
}

#[derive(Debug, Deserialize, Serialize)]
struct User {
    admin: u8,
    avatar_url: Option<String>,
    deactivated: u8,
    displayname: String,
    is_guest: u8,
    name: String,
    user_type: Option<String>,
}

fn get_server_config(server: Option<&str>) -> Result<ServerConfig, MatrixAPIError> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("matrixapirs")?;
    let config_path = xdg_dirs
        .find_config_file("config.toml")
        .ok_or_else(|| MatrixAPIError::ConfigFileError("config file does not exist".to_string()))?;
    let mut config_file = File::open(config_path)?;
    let mut config_string = String::new();
    config_file.read_to_string(&mut config_string)?;
    let config: Config = toml::from_str(&config_string)?;
    match server {
        Some(s) => match config.server.get(&s.to_string()) {
            Some(c) => Ok(c.clone()),
            None => Err(MatrixAPIError::ServerNotDefined(s.to_string())),
        },
        None => match config.server.get(&config.default_server) {
            Some(c) => Ok(c.clone()),
            None => Err(MatrixAPIError::ServerNotDefined(config.default_server)),
        },
    }
}

fn get_access_token(server_config: &ServerConfig) -> Result<String, MatrixAPIError> {
    let pass_cmd = Command::new("pass")
        .arg(server_config.pass_access_token.clone())
        .output()?;
    Ok(std::str::from_utf8(&pass_cmd.stdout)?.trim().into())
}

fn make_get_request(
    server_config: &ServerConfig,
    api_endpoint: &str,
    _query: Option<HashMap<String, String>>,
    access_token: Option<&str>,
) -> Result<reqwest::blocking::Response, MatrixAPIError> {
    let mut headers = header::HeaderMap::new();
    if let Some(a) = access_token {
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&*format!("Bearer {}", a)).unwrap(),
        );
    }
    let url = format!("{}/{}", server_config.server_url, api_endpoint);
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    Ok(client.get(&url).send()?)
}

pub fn get_server_version(server_name: Option<&str>, json: bool) -> Result<(), MatrixAPIError> {
    let server_config = get_server_config(server_name)?;

    let response = make_get_request(
        &server_config,
        "_synapse/admin/v1/server_version",
        None,
        None,
    )?;

    let version_map = response.json::<HashMap<String, String>>()?;

    if json {
        println!("{}", serde_json::to_string(&version_map).unwrap());
    } else {
        let table = table!(
            ["Python", version_map.get("python_version").unwrap()],
            ["Server", version_map.get("server_version").unwrap()]
        );
        table.printstd();
    };

    Ok(())
}

pub fn get_user_list(server_name: Option<&str>, json: bool) -> Result<(), MatrixAPIError> {
    let server_config = get_server_config(server_name)?;
    let access_token = get_access_token(&server_config)?;

    let response = make_get_request(
        &server_config,
        "_synapse/admin/v2/users?from=0&guests=true",
        None,
        Some(&access_token),
    )?;

    let status_code = response.status().as_u16();
    let user_list = match status_code {
        200 => response.json::<UserList>().unwrap(),
        _ => {
            let api_error_response = response.json::<APIErrorResponse>()?;
            return Err(MatrixAPIError::APIRequestError(APIErrorMessage::new(
                api_error_response,
                status_code,
            )));
        }
    };

    if json {
        println!("{}", serde_json::to_string(&user_list).unwrap());
    } else {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(row![
            "Name",
            "Displayname",
            "Admin",
            "Guest",
            "Deactivated",
            "User Type",
            "Avatar"
        ]);

        for user in user_list.users {
            let guest = match user.is_guest {
                0 => Cell::new("✘"),
                _ => Cell::new("✔"),
            };
            let deactivated = match user.deactivated {
                0 => Cell::new("✘"),
                _ => Cell::new("✔"),
            };
            let user_type = match user.user_type {
                Some(t) => t,
                None => "none".to_string(),
            };
            let avatar = match user.avatar_url {
                Some(s) => s,
                None => "none".to_string(),
            };
            match user.admin {
                0 => table.add_row(row![user.name, user.displayname, cFg->"✘", c->guest, c->deactivated, user_type, avatar]),
                _ => table.add_row(row![user.name, user.displayname, cFr->"✔", c->guest, c->deactivated, user_type, avatar]),
            };
        }

        table.printstd();
    }

    Ok(())
}
