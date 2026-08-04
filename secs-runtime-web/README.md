# secs-runtime-web

WebAssembly binding crate for using the SECS runtime from `secs-labo`.

## Build for secs-labo

Run this command from the `secs-runtime-web` directory:

```sh
wasm-pack build --target web --out-dir ../secs-labo/wasm/secs-runtime-web
```

This builds the wasm package and writes the generated files into:

```text
secs-labo/wasm/secs-runtime-web
```

If you run the command from the workspace root instead, use:

```sh
wasm-pack build ./secs-runtime-web --target web --out-dir ../secs-labo/wasm/secs-runtime-web
```
