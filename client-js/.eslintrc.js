module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    project: './tsconfig.json',
    tsconfigRootDir: __dirname,
    sourceType: 'module',
  },
  plugins: [
    '@typescript-eslint',
    'eslint-comments',
    'jest',
    'promise',
    'unicorn',
    'prettier',
  ],
  extends: [
    'airbnb-base',
    'airbnb-typescript/base',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
    'plugin:eslint-comments/recommended',
    // 'plugin:jest/recommended',
    // 'plugin:promise/recommended',
    // 'plugin:unicorn/recommended',
    'prettier',
  ],
  rules: {
    // 'no-console': 1,
    // 'prettier/prettier': 2,
    // // Too restrictive, writing ugly code to defend against a very unlikely scenario: https://eslint.org/docs/rules/no-prototype-builtins
    // 'no-prototype-builtins': 'off',
    // // https://basarat.gitbooks.io/typescript/docs/tips/defaultIsBad.html
    // 'import/prefer-default-export': 'off',
    // 'import/no-default-export': 'error',
    // // Use function hoisting to improve code readability
    // 'no-use-before-define': [
    //   'error',
    //   { functions: false, classes: true, variables: true },
    // ],
    // // Allow most functions to rely on type inference. If the function is exported, then `@typescript-eslint/explicit-module-boundary-types` will ensure it's typed.
    // '@typescript-eslint/explicit-function-return-type': 'off',
    // '@typescript-eslint/no-use-before-define': [
    //   'error',
    //   { functions: false, classes: true, variables: true, typedefs: true },
    // ],
    // // Common abbreviations are known and readable
    // 'unicorn/prevent-abbreviations': 'off',
    // // Airbnb prefers forEach
    // 'unicorn/no-array-for-each': 'off',
    // // It's not accurate in the monorepo style
    // 'import/no-extraneous-dependencies': 'off',
    // '@typescript-eslint/no-misused-promises': [
    //   'error',
    //   { ignoreIIFE: true }
    // ],
    // // Temp disabled because of missing types from external libs
    // "@typescript-eslint/no-unsafe-call": "off",
    // "@typescript-eslint/no-unsafe-call": "off"
  },
};
