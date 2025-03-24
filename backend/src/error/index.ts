import express, { Request, Response } from "express";
import { validationResult } from "express-validator";

// Error handling for validation
export const validate = (
  req: Request,
  res: Response,
  next: express.NextFunction
) => {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    return res.status(400).json({ errors: errors.array() });
  }
  next();
};
