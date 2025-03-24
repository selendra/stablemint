import { Request, Response, NextFunction } from "express";
import jwt from "jsonwebtoken";
import config from "../config";

interface DecodedToken {
  userId: string;
}

export const authMiddleware = (
  req: Request,
  res: Response,
  next: NextFunction
) => {
  // Get token from header
  const token = req.header("Authorization")?.replace("Bearer ", "");

  // Check if token exists
  if (!token) {
    return res.status(401).json({ message: "No token, authorization denied" });
  }

  try {
    // Verify token
    const decoded = jwt.verify(token, config.jwtSecret) as DecodedToken;

    // Add user from payload to request
    (req as any).user = decoded;
    next();
  } catch (error) {
    res.status(401).json({ message: "Token is not valid" });
  }
};
