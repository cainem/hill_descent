# Hill Descent – WASM demo (no graphics)

This directory contains the minimal harness for proving that the Rust crate can
be compiled to WebAssembly and called from JavaScript.

## 1. Build the WebAssembly package

```bash
wasm-pack build --target web --out-dir web/pkg
```

* `--target web` – generate ES-module JS wrappers suitable for direct import in
  the browser.
* `--out-dir web/pkg` – place the generated files next to `index.html`.

## 2. Serve the files

Any static file server will do. For example:

```bash
npx http-server ./web -c-1 -o
```

or with Python:

```bash
python -m http.server --directory web 8000
```

## 3. Open the demo

Navigate to `http://localhost:8000` (or whatever port your server printed). Open
the browser console; you should see the line:

```
himmelblau(3, 2) = 26
```

and the page displays the same text. This confirms JavaScript successfully
called the Rust `himmelblau` function compiled to WASM.

## Next steps

* Export `init_world`, `step_world` wrappers from Rust so JS can drive the full
  simulation.
* Swap the placeholder console output for D3 visualisation.
