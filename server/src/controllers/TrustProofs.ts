import {Request, Response, NextFunction} from 'express';
import { TrustProofsService } from '../services/TrustProofs';
import { writeJson } from '../utils/writer';

export class TrustProofsController {
    public async createTrustProof(req: Request, res: Response, next: NextFunction) {
    const trustProof = new TrustProofsService(); // req.body
    //   await trustProof.save();
      writeJson(res, "Create trust proof!", 200);
      // res.status(200).send("Create trust proof");
    }
}