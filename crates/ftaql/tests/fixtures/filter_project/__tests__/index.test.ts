import { main } from '../src/index';

describe('main', () => {
  it('returns greeting', () => {
    expect(main()).toBe('Hello, world!');
  });
});
