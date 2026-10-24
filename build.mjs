// Build script for Schwarzwasser.
// Bundles src/main.js with esbuild into dist/ and copies index.html + public/.
// In CI (GitHub Actions) PlayCanvas comes from npm. For offline builds, set
// PC_ENGINE_SRC=/path/to/engine/src to bundle the engine from its source checkout.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const dist = path.join(root, 'dist');
const watch = process.argv.includes('--watch');

let esbuild;
try {
    esbuild = (await import('esbuild')).default;
} catch {
    const alt = process.env.ESBUILD_PATH;
    if (!alt) throw new Error('esbuild not found: run npm install or set ESBUILD_PATH');
    esbuild = (await import(pathToFileURL(alt).href)).default;
}

// --- optional: build PlayCanvas from a source checkout --------------------------------------
const engineSrc = process.env.PC_ENGINE_SRC;
// The offline build keeps debug blocks: stripping them safely needs the engine's own toolchain.
const KEEP_DEBUG = true;
const enginePlugin = {
    name: 'engine-src',
    setup(build) {
        if (!engineSrc) return;
        build.onResolve({ filter: /^playcanvas$/ }, () => ({ path: path.join(engineSrc, 'index.js') }));
        build.onResolve({ filter: /^(fflate|meshoptimizer)$/ }, args => ({ path: args.path, namespace: 'stub' }));
        build.onLoad({ filter: /.*/, namespace: 'stub' }, () => ({ contents: 'export const zipSync=()=>{};export const strToU8=()=>{};export const MeshoptDecoder={};', loader: 'js' }));
        build.onLoad({ filter: /\.js$/ }, async (args) => {
            if (!args.path.startsWith(engineSrc)) return undefined;
            let src = await fs.promises.readFile(args.path, 'utf8');
            src = src.replace(/device\.(push|pop)Marker\(/g, 'device.$1Marker?.(').replace('$_CURRENT_SDK_VERSION', '2.23.3').replace('$_CURRENT_SDK_REVISION', 'src');
            // strip "// #if _DEBUG" / "// #if _PROFILER" blocks (release build behaviour)
            const out = [];
            const stack = [];
            for (const line of src.split('\n')) {
                const t = line.trim();
                let m;
                if ((m = /^\/\/ #if (_DEBUG|_PROFILER)$/.exec(t))) { stack.push(KEEP_DEBUG); continue; }
                if (/^\/\/ #else$/.test(t) && stack.length) { stack[stack.length - 1] = !stack[stack.length - 1]; continue; }
                if (/^\/\/ #endif$/.test(t) && stack.length) { stack.pop(); continue; }
                out.push(stack.every(Boolean) ? line : '');
            }
            return { contents: out.join('\n'), loader: 'js' };
        });
    }
};

const options = {
    entryPoints: [path.join(root, 'src/main.js')],
    bundle: true,
    format: 'esm',
    target: 'es2022',
    outfile: path.join(dist, 'game.js'),
    minify: !watch,
    sourcemap: watch,
    pure: ['Debug.assert', 'Debug.warn', 'Debug.warnOnce', 'Debug.log', 'Debug.logOnce', 'Debug.call',
        'Debug.deprecated', 'Debug.error', 'Debug.errorOnce', 'Debug.trace', 'DebugHelper.setName',
        'DebugHelper.setLabel', 'DebugGraphics.pushGpuMarker', 'DebugGraphics.popGpuMarker'],
    plugins: [enginePlugin],
    logLevel: 'info'
};

function copyStatic() {
    fs.mkdirSync(dist, { recursive: true });
    fs.copyFileSync(path.join(root, 'index.html'), path.join(dist, 'index.html'));
    fs.copyFileSync(path.join(root, 'style.css'), path.join(dist, 'style.css'));
    const pub = path.join(root, 'public');
    if (fs.existsSync(pub)) fs.cpSync(pub, dist, { recursive: true });
    fs.writeFileSync(path.join(dist, '.nojekyll'), '');
}

copyStatic();
if (watch) {
    const ctx = await esbuild.context(options);
    await ctx.watch();
    console.log('watching…');
} else {
    await esbuild.build(options);
}
