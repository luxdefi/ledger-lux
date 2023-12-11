# Ledger Lux app
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![npm version](https://badge.fury.io/js/%40zondax%2Fledger-lux-app.svg)](https://badge.fury.io/js/%40zondax%2Fledger-lux-app)
[![GithubActions](https://github.com/ava-labs/ledger-lux/actions/workflows/main.yaml/badge.svg)](https://github.com/ava-labs/ledger-lux/blob/main/.github/workflows/main.yaml)

---

![zondax_light](docs/zondax_light.png#gh-light-mode-only)
![zondax_dark](docs/zondax_dark.png#gh-dark-mode-only)

_Please visit our website at [zondax.ch](https://www.zondax.ch)_

---

This project contains the Lux app for Ledger Nano S, S+ and X.

- Ledger Nano S/S+/X BOLOS app
- Specs / Documentation
- Rust unit/integration tests
- Zemu tests

## ATTENTION

The app releases of this repository and the binaries you can build yourself with this repo
are considered unvetted development releases, use with caution.

The releases provided by Ledger via Ledger Live have undergone Ledger's security assessment
and thus are safe to use with real funds.

If you wish to use a development release, we recommend the following:
- Do not use in production
- Do not use a Ledger device with funds for development purposes
- Do use a separate and marked device that is used ONLY for development and testing

Nevertheless, this disclaimer does not apply to the client libraries provided in this repository.

## Download and install

*Once the app is approved by Ledger, it will be available in their app store (Ledger Live).
You can get development builds generated by our CI from the release tab. THESE ARE UNVETTED DEVELOPMENT RELEASES*

Download a release from here (https://github.com/ava-labs/ledger-lux/releases). You only need `installer_s.sh`

If the file is not executable, run
```sh
chmod +x ./installer_s.sh
```

then run:

```sh
./installer_s.sh load
```

# Development

## Dependencies

- Required libraries
  - If you are using Ubuntu: 
    ```sh
    sudo apt update && apt-get -y install build-essential git wget cmake \
    libssl-dev libgmp-dev autoconf libtool
    ```
   
- Docker CE for building
  - Instructions can be found here: https://docs.docker.com/install/

- `node > v14.0` for integration tests
  - We typically recommend using `n` or `nvm`

- A valid `rust` toolchain for unit tests
  - Automatic CI tests against 1.63

- Python 3 to sideload on your device

- Be sure you get the SDK and other dependencies:
  - If you have `just` installed you can use it (recommended):
    ```sh
    just init 
    ```
  - If not, you can use `make`:
    ```sh
    make init
    ```

## How to build ?

> We like clion or vscode but let's have some reproducible command line steps
>

- Building the app itself

  If you installed what is described above, just run:
    ```sh
    make
    ```

## Running tests

- Running rust tests (x64)

    If you just wish to run the rust unit and integration tests, just run:
    ```sh
    make rust_test
    ```
    ** Requires a rust toolchain available **

- Running device emulation+integration tests!!

   ```sh
    Use Zemu! Explained below!
    ```

- Running everything

  If you don't want to bother typing all those make commands by hand, you can skip them all!

  The only command you have to type is:
  ```sh
  make test_all
  ```

  This will initially run unit and integration tests (needs `rust` installed!), then install Zemu for you,
  clean the app's build files in case there's anything, proceed to build both application types
  and finally run the Zemu test suite.

## How to test with Zemu?

> What is Zemu?? Glad you asked!!
>
> Zemu is Zondax's testing+emulation framework for Ledger apps.
>
> Npm Package here: https://www.npmjs.com/package/@zondax/zemu
>
> Repo here: https://github.com/Zondax/zemu

Let's go! First install everything:

```sh
make zemu_install
```

Then you can run our Typescript based tests:

```sh
make zemu_test
```

To run a single specific test:

> At the moment, the recommendation is to run from the IDE. Remember to run `make build` if you change the app.

``` sh
cd zemu
yarn test -t 'test name'
```

This will run just the test maching the name provided

## How to debug a ledger app?

You can use vscode or clion to debug the app. We recommend using CLion but we provide a vscode (unsupported) configuration too.

### Preconditions

If you are using CLion, you need to a configuration file in your home directory: `$HOME/.gdbinit` with the following content:

```
set auto-load local-gdbinit on
add-auto-load-safe-path /
```

### Warnings

There are a few things to take into account when enabling Ledger App debugging:

- Once you enable the local .gdbinit that is located in your project workspace. You **will break** local Rust debugging in your host. The reason is that debugging unit tests will use the same `.gdbinit` configuration that sets the environment to ARM. We are looking at some possible fixes. For now, if you want to debug unit tests instead of the ledger app, you need to comment out the lines in `.gdbinit`

### Debugging

1. Build your app

    ```sh
    make
    ```

2. Define your debug scenario

    Open `tests/zemu/tools/debug.mjs` and look for the line:

    ```sh
    /// TIP you can use zemu commands here to take the app ...
    ```

    You can adjust this code to get the emulator to trigger a breakpoint in your app:
    - send clicks
    - send APDUs, etc

3. Launch the emulator in debug mode

    > If you didnt install Zemu yet (previous section), then run `make zemu_install`

    ```sh
    make zemu_debug
    ```

    The emulator will launch and immediately stop. You should see a light blue window

4. Configure Clion debugger

    Your configuration should look similar to this:

     ![image](docs/img/clion_debugging.png)

    Check that the path mappings are correct

5. Start CLion debugger

    You will hit a breakpoint in main.
    Add breakpoints in other places and continue.

    Enjoy :)

## Using a real device

### How to prepare your DEVELOPMENT! device:

>  You can use an emulated device for development. This is only required if you are using a physical device
>
>    **Please do not use a Ledger device with funds for development purposes.**
>
>    **Have a separate and marked device that is used ONLY for development and testing**

   There are a few additional steps that increase reproducibility and simplify development:

**1 - Ensure your device works in your OS**
- In Linux hosts it might be necessary to adjust udev rules, etc.

  Refer to Ledger documentation: https://support.ledger.com/hc/en-us/articles/115005165269-Fix-connection-issues

**2 - Set a test mnemonic**

Many of our integration tests expect the device to be configured with a known test mnemonic.

- Plug your device while pressing the right button

- Your device will show "Recovery" in the screen

- Double click

- Run `make dev_init`. This will take about 2 minutes. The device will be initialized to:

   ```
   PIN: 5555
   Mnemonic: equip will roof matter pink blind book anxiety banner elbow sun young
   ```

### 3. Add a development certificate

- Plug your device while pressing the right button

- Your device will show "Recovery" in the screen

- Click both buttons at the same time

- Enter your pin if necessary

- Run `make dev_ca`. The device will receive a development certificate to avoid constant manual confirmations.

## Building the Ledger App

### Loading into your development device

The Makefile will build the firmware in a docker container and leave the binary in the correct directory.

- Build

   ```sh
   make                # Builds the app
   ```

- Upload to a device

   The following command will upload the application to the ledger:

   _Warning: The application will be deleted before uploading._
   ```sh
   make load          # Loads the built app to the device
   ```

## APDU Specifications

- [APDU Protocol](./docs/APDUSPEC.md)
