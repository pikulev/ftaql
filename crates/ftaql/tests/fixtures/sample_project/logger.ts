import { subtract } from "./utils";

const logger = {
  log: (message: string) => {
    console.log(message, subtract(2, 1));
  },
};

export default logger; 