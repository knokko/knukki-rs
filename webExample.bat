wasm-pack build examples/%1 --target web --out-name web --out-dir ../../pkg

cargo install basic-http-server
start "" http://localhost:4000/pkg/
basic-http-server .