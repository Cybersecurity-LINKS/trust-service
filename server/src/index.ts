import express, { Express, Request, Response } from 'express';
import dotenv from 'dotenv';
import router from './routes/Api';


dotenv.config();

const app: Express = express();
const port = process.env.PORT;


app.get('/', (req: Request, res: Response) => {
  res.send('Hello world!');
});

//route methods
app.use('/api', router);

app.listen(port, () => {
  console.log(`⚡️[server]: Server is running at http://localhost:${port}`);
});