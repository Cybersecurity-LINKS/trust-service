# trust-service

```shell
cd server
npm init
npm install --save-dev typescript ts-node
npm install --save-dev @types/node @types/express
npx tsc --init
npm install express dotenv
npm install -D concurrently nodemon
```

- edit `rootDir` and `outDir` in `tsconfig.json` 
- add to `package.json`
    ```json 
    "build": "npx tsc"
    "start": "node dist/index.js" 
     "dev": "concurrently \"npx tsc --watch\" \"nodemon -q dist/index.js\""
    ```
Commands for building the app’s container image and starting the app container:
```shell
docker build -t ts .
docker run -dp 8000:8000 ts  
```

https://developer.mozilla.org/en-US/docs/WebAssembly/Rust_to_Wasm
https://rustwasm.github.io/docs/wasm-bindgen/web-sys/using-web-sys.html

```shell
cargo watch -x run
```
