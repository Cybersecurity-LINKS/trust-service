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
    