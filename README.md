# matrixapirs

A tool to use the synapse API written in rust

## Dependencies

-   [pass](https://www.passwordstore.org/)

## Config

The config file is read from `$XDG_CONFIG_HOME/matrixapirs/config.toml`, which
should default to `~/.config/matrixapirs/config.toml`.

Here is an example config file:

```toml
default_server = "server1"

[server.server1]
server_name = "matrix.server1.de"
server_url = "https://matrix.server1.de:8448"
pass_access_token = "server1/access_token"

[server.server2]
server_name = "matrix.server2.de"
server_url = "https://matrix.server2.de:8448"
pass_access_token = "server2/access_token"
```

The meaning of each config setting is as follows:

| setting           | meaning                                                                 |
| ----------------- | ----------------------------------------------------------------------- |
| default_server    | The subtable of the sever table to be used when now server is specified |
| sever_name        | Name of the server, used to create user ids                             |
| server_url        | The URL to the server                                                   |
| pass_access_token | The argument to retrieve the access token from pass                     |

## Access Token

To get the access token for a given server, pass is used. So in order to use
`matrixapirs`, the token must be added to pass and the location must be set in
`pass_access_token` in the configuration for the server.
