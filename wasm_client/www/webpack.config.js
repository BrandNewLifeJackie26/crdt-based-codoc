const path = require('path');
const webpack = require('webpack');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');
const WebpackRequireFrom = require('webpack-require-from');

const APP_DIR = path.resolve(__dirname, './src');
const BUILD_DIR = path.resolve(__dirname, './build');
const MONACO_DIR = path.resolve(__dirname, './node_modules/monaco-editor');

module.exports = (env, argv) => {
    return {
        mode: 'none',
        target: 'web',
        devtool: 'source-map',
        resolve: {
            modules: [path.resolve(__dirname, './src'), 'node_modules'],
            extensions: ['.ts', '.tsx', '.js', '.mjs', '.css'],
            alias: {
                vscode: require.resolve(
                    'monaco-languageclient/lib/vscode-compatibility'
                ),
                crypto: false,
                net: false,
                os: false,
                path: false,
            },
        },
        entry: {
            app: path.join(APP_DIR, 'index.js'),
        },
        output: {
            filename: '[name].[contenthash].js',
            path: BUILD_DIR,
        },
        experiments: {
            asyncWebAssembly: true,
        },
        module: {
            rules: [
                {
                    test: /\.tsx?/,
                    use: [
                        {
                            loader: 'ts-loader',
                            options: {
                                transpileOnly: true,
                                happyPackMode: true,
                            },
                        },
                    ],
                    exclude: /node_modules/,
                },
                {
                    test: /\.(js|jsx)$/,
                    exclude: /node_modules/,
                    use: [
                        {
                            loader: 'babel-loader',
                            options: {
                                presets: [
                                    '@babel/preset-env',
                                    '@babel/preset-react',
                                ],
                                plugins: [
                                    '@babel/plugin-proposal-class-properties',
                                ],
                            },
                        },
                    ],
                },
                {
                    test: /\.css$/,
                    include: APP_DIR,
                    use: [
                        {
                            loader: 'style-loader',
                        },
                        {
                            loader: 'css-loader',
                        },
                    ],
                },
                {
                    test: /\.css$/,
                    include: MONACO_DIR,
                    use: ['style-loader', 'css-loader'],
                },
            ],
        },
        devServer: {
            contentBase: BUILD_DIR,
            compress: true,
            historyApiFallback: true,
            port: 9000,
        },
        plugins: [
            new CleanWebpackPlugin({
                cleanOnceBeforeBuildPatterns: [
                    path.join(process.cwd(), 'build/**/*'),
                ],
            }),
            new HtmlWebpackPlugin({
                title: 'WebEditor',
                template: path.join(APP_DIR, 'index.html'),
                filename: 'index.html',
                excludeChunks: ['notFound'],
            }),
            new MonacoWebpackPlugin({
                languages: ['json', 'yaml'],
            }),
            new webpack.ProvidePlugin({
                process: 'process/browser',
            }),
            new CopyWebpackPlugin(['./src/index.html']),
        ],
    };
};
