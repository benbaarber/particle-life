/** @type {import('webpack').Configuration} */

const path = require("path");
const CopyPlugin = require("copy-webpack-plugin")
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: path.resolve(__dirname, "src", "App.tsx"),
    mode: "development",
    target: "web",
    module: {
        rules: [
            {
                test: /\.(ts|tsx)/,
                include: path.resolve(__dirname, "src"),
                exclude: /node_modules/,
                use: ["ts-loader"]
            },
            {
                test: /\.(css)/,
                include: path.resolve(__dirname, "src"),
                exclude: /node_modules/,
                use: ["style-loader", "css-loader", "postcss-loader"]
            },
        ]
    },
    resolve: {
        extensions: [".css", ".js", ".jsx", ".tsx", ".ts", ".cjs"],
        alias: {
            "tailwindcss/resolveConfig": "tailwindcss/resolveConfig.js",
            "~": path.resolve(__dirname)
        }
    },
    output: {
        path: path.resolve(__dirname, "../dist/client"),
        filename: "bundle.js",
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: path.resolve(__dirname, "index.html"), to: "index.html" },
            ]
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "../wasm"),
            forceMode: "production"
        }),
    ],
    devServer: {
        static: path.resolve(__dirname, "../dist/client"),
        port: 3000,
        historyApiFallback: true
      },
    experiments: {
        asyncWebAssembly: true
   }
};
