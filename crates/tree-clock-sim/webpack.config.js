const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: 'development',
  devtool: 'source-map',
  entry: [
      './web/index.ts'
  ],
  output: {
    path: dist,
    filename: "index.js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "static")
    ]),

    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: "--out-name tree-clock-sim-rs"
    }),
  ],
  module: {
        rules: [
            {
                test: /\.ts$/,
                use: 'awesome-typescript-loader'
            },
            {
                test: /\.wasm$/,
                type: "webassembly/experimental"
            },
            {
                test: /\.css$/,
                exclude: /[\/\\]web[\/\\]/,
                use: [
                    {
                        loader: 'style-loader'
                    },
                    {loader: 'css-loader'}
                ]
            }, {
                test: /\.css$/,
                exclude: /[\/\\](node_modules|bower_components|public)[\/\\]/,
                use: [
                    {
                        loader: 'style-loader',
                    },
                    {
                        loader: 'css-loader',
                        options: {
                            modules: true,
                            importLoaders: 1
                        }
                    }
                ]
            },
            {
                test: /\.(png|svg|jpg|gif)$/,
                use: ['file-loader']
            }
        ]
    },
    resolve: { extensions: [".web.ts", ".web.js", ".ts", ".js", ".wasm"] },
};
