# WeensyOS in Rust

## Dependencies

When building baremetal, we need to setUp correct target architecture to explain `cargo` how to compile the project. In out case WeensyOS is a x86_64 operating system.

```bash
rustup target add x86_64-unknown-none
```

```bash
rustup default nightly
```

```bash
cargo build --target x86_64-unknown-linux-gnu --release
```

## How to build

We modify the `Makefile` build and clean commands to add extra steps for the Rust compilation and correct linking. To start WeensyOS, run as usual `make run` or `make run-console`.

Note, WeensyOS is usually complitted in the Yale zoo environement. Here we let the students to install it locally if they have Linux machine, or use the `Dockerfile` with the provided documentation in the `devenv` folder. You can disable the locking of `qemu` being not killed on some zoo node by running command with `USE_HOST_LOCK=0` flag, or manually modifying the GNUMakefile to 0 (line 50).

## Documentation (Draft):

#### Setting Up the Environemnt

- Since we aim to keep the rest of the OS unchanged, we are limited to the C implementation logic linking, which means we want to produce the static `.o` files even in Rust. This is possible if we use `staticlib` crate type, which generates `.a` files that could be furter unpacked to `.o` (includes all other deps `.o` files managed by cargo). Linking such files is possible with Rust FFI (e.g. `unsafe extern "C"`, `#[repr(C)]` etc) which comes from Rust initially build of C LLVM.
- Another way would be to use `rustc` to get `.o` files drectly. However, this is bothersome b/c (being short) of manuall dependancy handling that cargo could do it for us automatically. Therefore we let the cargo do most of the job, and, at the end, create a `Makefile` shell script for unpacking Rust .a archives and moving to the `obj/` folder `.o` files. This could be optimized further by setting the LTO to "fat" in `Cargo.toml` that would remove all unecessary symbols not used in the project (meaning, all except the actual `.o` file). So the this approach is actually clean with the minimal amount of changes required.
- Therefore, as ultimate decision, I have created a Rust `workspace` with kernel, vm, kloader folders that have their own `Cargo.toml` files for the separate `.a` lib compilation. Otherwise, `cargo` will generate a single archive combined, so here we think about each of the file as a separate module.
- The other differnce between Rust-C project is the header files, which Rust does not handles. Thankfully, there are `binding` crates that I could use to quickly generate the necessary structs correctly - assserted with having same C-Rust memory layout, which is very helpful. Keeping such files unchanged, would make Rust complain about the camel_types, unsused definitions, etc, so to keep everything clean we lazy-move these structs on need to the new `bindings` modules folder next to the rest of the workspace, which is further imported as dependancy in `Cargo.toml`. 
- Building in a `no_std` environment is essential for the Operating System. However, establishing this macro is not required and *is expected* to be missing in the rust kernel, since that would otherwise require handling of some additional code e.g. local panic handlers (required by Rust), allocators, atomic counters etc. that is already provided by the local user system. Still, as long as we do not use the `std` library explicitly, it should allow us to surpass the compilation of necessary function, drop the `std` symbols with LTO and process functions normaly in the C environemnt without having redundant code in the rust kernel.

#### C-Rust Implementation Issues

- Importing one function from one rust file to another is complicated and should be used with extern C repr. This is because cargo would copy over the necessary dependancies and cause error of having double definition when loading `.o` object files that use/define same function.
- Static global variables are hard to handle. C implementation could easily define one without setting values immediately, but this would be `unsafe` in Rust. Therefore most of the times are required to add to the bindings the implementation for the Default traits. This could normally be handled automatically, but we are using `repr(C)` to ensure the correct memory layout, so this would not work for us anymore and adding some redundancy in the code in comparison to the orifinal C implementation of the WeensyOS.