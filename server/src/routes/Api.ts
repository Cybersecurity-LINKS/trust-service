import { Router } from 'express';
import { TrustProofsController } from '../controllers/TrustProofs';

const router = Router();
const trustProofsController = new TrustProofsController();

router.get('/trust-proofs', trustProofsController.createTrustProof);

export default router;