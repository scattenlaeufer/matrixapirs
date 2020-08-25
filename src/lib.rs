use reqwest::header;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

#[derive(Debug)]
pub enum MatrixAPIError {
    ServerNotDefined(String),
    ConfigFileError(String),
    AccessTokenError(String),
    ReqwestError(String),
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
                write!(f, "There was an error during the API request: {}", s)
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
) -> Result<HashMap<String, String>, MatrixAPIError> {
    let mut headers = header::HeaderMap::new();
    if let Some(a) = access_token {
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&*format!("Bearer {}", a)).unwrap(),
        );
    }
    println!("{:?}", &headers);
    let url = format!("{}/{}", server_config.server_url, api_endpoint);
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let response = client
        .get(&url)
        .send()?
        .json::<HashMap<String, String>>()
        .unwrap();
    Ok(response)
}

pub fn get_server_version(server_name: Option<&str>) -> Result<(), MatrixAPIError> {
    let server_config = get_server_config(server_name)?;

    let result = make_get_request(
        &server_config,
        "_synapse/admin/v1/server_version",
        None,
        None,
    )?;
    println!("{:?}", result);

    Ok(())
}

pub fn get_user_list(server_name: Option<&str>) -> Result<(), MatrixAPIError> {
    let server_config = get_server_config(server_name)?;
    let access_token = get_access_token(&server_config)?;

    let result = make_get_request(
        &server_config,
        "_synapse/admin/v2/users?from=0&guests=true",
        None,
        Some(&access_token),
    );

    println!("{:?}", result);

    Ok(())
}
