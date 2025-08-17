# `Tondi Dashboard`


[<img alt="github" src="https://img.shields.io/badge/github-aspectron/kaspa--ng-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/aspectron/kaspa-ng)
<img src="https://img.shields.io/badge/platform-native-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/github/actions/workflow/status/aspectron/kaspa-ng/ci.yaml?branch=master&style=for-the-badge" height="20">

<p align="center" style="margin:32px auto 0px auto;text-align:center;font-size:10px;color:#888;">
<img src="https://aspectron.org/images/projects/kaspa-ng-screen-01.png" style="display:block;max-height:320px;max-width:524px;width:524px;height:auto;object-fit:cover;margin: 0px auto 0px auto;"><br/><sup>RUSTY KASPA P2P NODE &bull; KASPA WALLET &bull; BLOCKDAG VISUALIZER</sup></p>

## Features

This software incorporates the following functionality:
- Rusty Tondi p2p Node
- Tondi wallet based on the Rusty Tondi SDK
- Rusty Tondi CLI wallet
- BlockDAG visualizer
- Remote node connectivity

This project is built on top of and incorporates the [Rusty Tondi](https://github.com/tondinet/rusty-tondi) core framework.

This software is ideological in nature with a strong focus on architecture and decentralization. It is a unified codebase tightly coupled with the Rusty Tondi project. Fully written in Rust, it is available as a high-performance desktop application on all major operating systems (Windows, Linux and MacOS) as well as in major web browsers. It does not rely on any JavaScript or Web frameworks, which greatly strengthens its security profile. The Web Browser extension based on this infrastructure is currently under development.

You can find more information about this project at [https://tondinet.org/en/projects/tondi-dashboard.html](https://tondinet.org/en/projects/tondi-dashboard.html).

## Releases

- You can obtain the latest binary redistributables from the [Releases](https://github.com/tondinet/tondi-dashboard/releases) page.
- You can access the official Web App online at [https://tondi-dashboard.org](https://tondi-dashboard.org).

## Building

To build this project, you need to be able to build Rusty Tondi. If you have not built Rusty Tondi before, please follow the Rusty Tondi [build instructions](https://github.com/tondinet/rusty-tondi/blob/master/README.md).

In addition, on linux, you need to install the following dependencies:

#### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install libglib2.0-dev libatk1.0-dev libgtk-3-dev librust-atk-dev
```

#### Fedora:
```bash
sudo dnf install glib2-devel atk-devel gtk3-devel
```

Once you have Rusty Tondi built, you will be able to build and run this project as follows:

### From GitHub repository:

#### Running as Native App
```bash
cargo run --release
```

#### Running as Web App
```bash
cargo install trunk
trunk serve --release
```
Access via [https://localhost:8080](https://localhost:8080)

While the application is a static serve, you can not load it from the local file system due to CORS restrictions. Due to this, a web server is required. This application is designed to be built with [Trunk](https://trunkrs.dev/) and is served from the `dist/` folder.  This is a self-contained client-side application - once the application is loaded, the web server is no longer required.

#### Running Headless

Tondi Dashboard application binary can be started in 3 ways:
- `tondi-dashboard` - starts Tondi Dashboard in the default desktop mode
- `tondi-dashboard --daemon [rusty-tondi arguments]` - starts Tondi Dashboard as a Rusty Tondi p2p node daemon
- `tondi-dashboard --cli` - starts Tondi Dashboard as a Rusty Tondi CLI wallet

If you need access to the wallet in a headless environment, you can start Tondi Dashboard in daemon mode and then use the CLI wallet to access the wallet.

#### Software Rendering for Windows x64 VMs

Tondi Dashboard uses OpenGL.  Due to that, Tondi Dashboard may have problems powering up on the legacy hardware or inside of virtualization platforms that do not support hardware acceleration.

To address this, you can use Mesa 3d Software Emulation library.
Mesa 3d library build is available for download from the `resources/windows/mesa3d` folder of this repository.  
Simply extract the archive and place the `opengl32.dll` file in the same folder as the `tondi-dashboard.exe` executable.

This library build was placed in this repository for direct download on 2025-05-14.
The original build was created by Federico Dossena at [https://fdossena.com/?p=mesa/index.frag](https://fdossena.com/?p=mesa/index.frag).

#### Solo Mining

You can use the following stratum bridge to solo mine with Tondi Dashboard: https://github.com/rdugan/tondi-stratum-bridge/releases
In order to allow for mining, you need to enable gRPC interface in the Settings panel (*'Local'* if running the stratum bridge on the same machine, *'Any'* if running the stratum bridge on a different machine).
In the stratum configuration setup tondi_address of the machine running Tondi Dashboard (`127.0.0.1` if local) and use `stratum+tcp://<stratum bridge ipv4 address>:(5555)`.

## License

Licensed under the [ISC License](LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any
additional terms or conditions.

## Donations

If you are a Tondi investor, please consider supporting this project. The funds will be used to cover operational costs and further the project's functionality. 

`tondi:qq2efzv0j7vt9gz9gfq44e6ggemjvvcuewhzqpm4ekf4fs5smruvs3c8ur9rp`
