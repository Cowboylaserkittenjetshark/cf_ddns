# cf_ddns
This should not be used in its current state, I only started it as a project to try out Rust with. I may rewrite it in the future if I have time.
## Usage
`cf_ddns --config <path_to_config_file>`
## Configuration
```toml
# Cloudflare specific settings
[cloudflare]
token = "<api_token>"                # The api token to use
domains = [                          # Zones to update. Can either be the zone's id or its name
  "023e105f4ecef8ad9ca31a8372d0c353",   # Specifying a zone by its id. The id can be fetched from the dashboard
  "example.com",                        # Specifying a zone by its name. This is just the domain name
]
tag = "!ddns"                        # Only records that contain this string in their comment field or their tag array will be updated

# Settings specific to fetching public ip addresses
[fetchers.v4]                        # Settings for fetching an ipv4 address
type = "Nest"                          # The fetcher to use
router_ip = "192.168.86.1"
[fetchers.v6]                        # Settings for fetching an ipv6 address
type = "Disabled"
```
### API Token
Cloudflare API tokens can be generated from the dashboard. (See [Cloudflare Docs](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/))

The API token needs the following `Permissions`:
| Field 1 | Field 2| Field 3 |
:--------:|:------:|:--------:
| Zone    | Zone   | Read    |
| Zone    | DNS    | Read    |
| Zone    | DNS    | Edit    | 

`Zone Resources` should include any zone you want to update
### Fetchers
#### Nest
Fetches the public IP from supported routers using [this API.](https://github.com/olssonm/google-wifi-api)

Supports: IPv4 only

Options:
| Option | Description |
:-------:|:------------:
|router_ip| Local ip of the router|
#### Ipify
Fetches the public IP from [ipify.org](https://ipify.org)

Supports: IPv4 and IPv6

Options: None
#### Disabled
Does not fetch an IP. Corresponding records will be skipped.

Supports: IPv4 and IPv6

Options: None
## Scheduling
There is no built in daemon or scheduler. Most systems already use systemd timers or have a cron daemon.
