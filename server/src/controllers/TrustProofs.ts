import {Request, Response} from 'express';

export class TrustProofsController {
    public async createTrustProof(req: Request, res: Response) {
    //   const trustProof = new TrustProof(req.body);
    //   await trustProof.save();
      res.status(200).send("Create trust proof");
    }
}