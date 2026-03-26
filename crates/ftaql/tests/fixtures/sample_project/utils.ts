import logger from "./logger";

export const add = (a: number, b: number) => {
  logger.log(`adding ${a} and ${b}`);
  return a + b;
};

export function subtract(a: number, b: number): number {
  return a - b;
}

export function multiply(a: number, b: number): number {
    logger.log('Multiplying numbers');
    return a * b;
} 