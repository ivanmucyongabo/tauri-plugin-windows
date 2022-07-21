require("esbuild")
.build({
    logLevel: "info",
    absWorkingDir: __dirname,
    entryPoints: ["src/main.ts"],
    loader: {".ts": "ts"},
    bundle: true,
    minify: true,
    sourcemap: true,
    format: 'iife',
    tsconfig: 'tsconfig.json',
    target: ['chrome58', 'firefox57', 'safari11', 'edge16'],
    outfile: "public/bundle.js",
})
.catch(() => process.exit(1));