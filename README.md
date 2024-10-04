# rpu-qafm

Firmware of the feedback engine for the project quantum-limited atomic force microscopy
[QAFM](https://qafm.eu/). The firmware runs on a core of the real-time processing unit (RPU) of a
Presto unit.

## Notable files

- `do.sh`  
    A helper Bash script to run commands through a dedicated container. See section *"Setup - Using
    a container"* below.
- `src/user.rs`  
    The Rust source file containing the user logic for the RPU core.
- `target/armv7r-none-eabihf/release/qafm`  
    The compiled firmware for the RPU core.
- `examples/lockin_feedback.py`  
    Example python script using the firmware functionality.

## Setup

### Using a container

We recommend using [podman](https://podman.io/) to manage your containers. It's free, open source,
fast, and safe with no admin privileges required. If for any reason you can't or don't want to use
podman, you can also use the more popular [docker](https://www.docker.com/). You should be able to
just replace `TOOL="/usr/bin/podman"` with `TOOL="/usr/bin/docker"` in the file `do.sh`.

1. Build the container image (just once):
    ```
    ./do.sh build
    ```

1. Run the container:
    ```
    ./do.sh run
    ```
    this will get you a `bash` prompt. You can run any other command by adding it to the end. For
    example:
    ```
    ./do.sh run cargo check
    ```
    will run `cargo check` on the Rust project and check that all is great.

### Using a local toolchain

If you don't want to use a container-based solution, you can set up your local machine to cross
compile Rust to the `armv7r` target. At minimun, you need to:

1. [Install Rust](https://www.rust-lang.org/tools/install)
1. Install the target:
    ```
    rustup target add armv7r-none-eabihf
    ```

You might also need to install a linker able to target the `armv7r` platform.

## Compiling

Once you are set up, compiling is easy:
```
./do.sh run cargo build -r
```
or just `cargo build -r` if you use the local-toochain setup.

The compiled RPU firmware will be in `target/armv7r-none-eabihf/release/qafm`.

## License

Licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
  (see [LICENSE-APACHE](LICENSE-APACHE))

* [MIT License](https://opensource.org/licenses/MIT)
  (see [LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
