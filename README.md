# udpproxy

This project is a fork of neosmart/udpproxy.

The 2 main goals of this fork are to :

 - make this proxy more Docker/Kubernetes friendly by moving all cofiguration to Env variables instead of Shell arguments

 - add `hooks` that will allow execution of custom functions at different point of the re-routing process

## Usage

`udpproxy` is a command-line application. One instance of `udpproxy` should be started for each remote endpoint you wish to proxy data to/from. All configuration is done via environment variables.

```
Variables used strictly for the Proxy :

    - *LOCAL_PORT:                   The local port to which udpproxy should bind to
    - *REMOTE_PORT:                  The remote port to which UDP packets should be forwarded
    - *REMOTE_HOST:                  The remote address to which packets will be forwarded
    - *BIND_ADDR:                    The address on which to listen for incoming requests
    - DEBUG:                         Enable debug mode

Variables used for `hooks`

    - *URI:                          
    - *TOKEN:                        
    - *SERVICE_ID:                   
    - TIME_BEFORE_STOP:             
    - TIME_TICK_CHECK_STOP:         
    - TIME_WAIT_STATUS_CHANGE:      
    - TIME_CHECK_ALREADY_STARTED:   
```

Where possible, sane defaults for arguments are provided automatically.

## Installation

A docker image is published on docker-hub :

    godwin42sh/udpproxy
