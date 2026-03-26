import { add, subtract } from './utils';
import logger from './logger';
import { Button } from '@/components/button';

function app() {
  logger.log('Inside app');
  const result = add(2, 2);
  const result2 = subtract(5, 3);
  logger.log(`2 + 2 = ${result}`);
  logger.log(`5 - 3 = ${result2}`);
  logger.log(Button());
}

export default app;