import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';

export default [
  {
    rules: {
      'prettier/prettier': [
        'warn',
        {
          singleQuote: true,
        },
      ],
    },
  },
  eslintPluginPrettierRecommended,
];
