const tailwindcss = require('tailwindcss');
const autoprefixer = require('autoprefixer');

const config = {
  plugins: [
    tailwindcss("./tailwind.config.cjs"),
    autoprefixer,
  ],
};

module.exports = config;
