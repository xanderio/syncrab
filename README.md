# SynCrab
The goal of this bot is to provide an in matrix interface to the 
[synapse admin API](https://matrix-org.github.io/synapse/latest/usage/administration/admin_api/index.html)


## Installation

```sh
cargo install --path .
```

## Configuration

The configuration is stored in file called `config.toml`. SynCrab will currently only look in the the folder where it was started from.

```toml
[synapse]
# Url of the Homeserver on which you want to use this bot
url = "https://example.org"
# user this bot will use for communication and executing admin api action
# this user needs to be an admin in syapse
user = "@user:example.org"
# password of the user
password = "password" 

[store]
# on disk location where SynCrab will place it's state and crypto store
location = "./store"
# This password will be using to encrypt the store
password = "super secret password"

```
