require("esbuild")
.buildSync({
    logLevel: "info",
    absWorkingDir: __dirname,
    entryPoints: ["index.ts"],
    entryNames: '[dir]/[name]',
    loader: {".ts": "ts"},
    minify: true,
    format: 'esm',
    tsconfig: 'tsconfig.json',
    outdir: '../webview-dist',
});