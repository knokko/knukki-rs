wasm-pack build examples/%1 --target web --out-name web --out-dir ../../pkg

start "" http://localhost:8000/pkg/
python3 -m http.server || python -m http.server