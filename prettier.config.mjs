/** @type {import("prettier").Config} */
export default {
  useTabs: false,
  trailingComma: 'all',
  printWidth: 80,
  plugins: [
    'prettier-plugin-astro',
    'prettier-plugin-tailwindcss',
  ],
  overrides: [
    {
      files: '*.astro',
      options: {
        parser: 'astro',
      },
    }
  ],
};
