# zexec

**zexec** allows users to safely execute remote commands on a server. It relies on [Zenroom] to
check that a certain user can execute a certain command. It does so by storing a Zenroom
contract and the user public keys; the user trigger the command by sending it along with
its signature. **zexec** will execute the Zenroom contract and, if the signature matches,
it will execute the command requested.

## Features

- Cryptographically secure by using [Zenroom]
- Small and static executable

## Getting started

### Binary relaese

There is no binary release yet, you need to compile zexec from the source code.

### Compiling from the source

zexec requires the nightly version of the Rust toolchain; [rustup] can be used to download and
setup Rust.

In addition, Zenroom requires the following dependencies:

- clang
- cmake
- linux-headers
- llvm
- meson
- xxd (usually contained in the vim package)
- zsh

### Nightly rustc 

Some dependancies require a nightly or dev build of rust to run. An easy way to solve this is:

```bash
rustup default nightly
```

After the dependencies have been installed and rust set to nightly, run the following command:

```bash
cargo build --release
```

## Usage

**zexec** looks by default in the folder */etc/zexec*, as defined [here](https://github.com/dyne/zexec/blob/main/src/main.rs#L33) and saves output into */var/lib/zexec/logs/*. Create both folders and copy the "command" smart contract to */etc/zexec*

```bash
sudo mkdir /etc/zexec
sudo mkdir -p /var/lib/zexec/logs/
sudo cp ./contracts/* /etc/zexec
```


To run zexec:

```bash
sudo ./target/release/zexec
```

It will by default listen on the port 9856 on the localhost address. You can change it by
respectively using `-p` and `-a` in the command line.

Then you can pass it data via a POST, for example using curl: 

```bash
curl -X POST "http://127.0.0.1:9856/contracts/command" -H  "accept: application/json" -H  "Content-Type: application/json" -d "{\"Alice\":{\"signature\":{\"r\":\"5n2RrZFIxuQtETEfKfQwPhR7FvW4hfxOGn2bQOz69gs=\",\"s\":\"vZMggPH3U2A30IwwzTUpLRLNVeUoUjWlG3/wDZaOXbM=\"}},\"command\":\"ls\"}"
```

Zexec should then return the name of the file, where it logged the output of the command passed in the POST (a simple "ls" in this case), which you can find in the folder */var/lib/zexec/logs/*.

## License

zexec is written and maintained by Jaromil Rojo ([@jaromil]) and Danilo Spinella ([@danyspin97]).
It is licensed under the [GPL-3.0].

[Zenroom]: https://github.com/dyne/Zenroom
[rustup]: https://rustup.rs/
[@jaromil]: https://github.com/jaromil
[@danyspin97]: https://github.com/danyspin97
[GPL-3.0]: https://www.gnu.org/licenses/gpl-3.0.en.html
