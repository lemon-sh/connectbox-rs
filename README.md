# connectbox-rs
API client library for the Compal CH7465LG, which is a cable modem provided by various European ISPs under the name Connect Box.

For more information, see the crate documentation.

### Shell
There is a very work-in-progress shell available in the connectbox-shell crate in this repository. I will include more information about it once it's in a usable state.

### IPv6 Notice
I am running my modem in the IPv4 mode, so the options available to me are different than what IPv6 mode users see. Thus, this crate will likely not work correctly with IPv6 mode Connect Boxes.

Contributions adding IPv6 support are always welcome, though.

### Credits
Special thanks to the authors of the following projects:
 * [home-assistant-ecosystem/python-connect-box](https://github.com/home-assistant-ecosystem/python-connect-box)
 * [ties/compal_CH7465LG_py](https://github.com/ties/compal_CH7465LG_py)

They have saved me from hours of reverse engineering work.
