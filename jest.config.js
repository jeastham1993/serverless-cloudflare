module.exports = {
    testEnvironment: 'node',
    roots: ['<rootDir>/src/backend'],
    testMatch: ['**/*.test.ts'],
    transform: {
      '^.+\\.tsx?$': 'ts-jest'
    }
  };