# connectbox-rs
API client library for the Compal CH7465LG, which is a cable modem provided by various European ISPs under the name Connect Box.

For more information, see the crate documentation.

## Supported endpoints
- [x] Devices list
- [x] Port forwarding
- [ ] Wireless settings

This list will grow as the project progresses.

### IPv6 Notice
I am running my modem in the IPv4 mode, so the options available to me are different than what IPv6 mode users see. Thus, this crate will likely not work correctly with IPv6 mode Connect Boxes.

Contributions adding IPv6 support are always welcome, though.

### Similar projects
* [home-assistant-ecosystem/python-connect-box](https://github.com/home-assistant-ecosystem/python-connect-box) (Python)
* [ties/compal_CH7465LG_py](https://github.com/ties/compal_CH7465LG_py) (Python)
