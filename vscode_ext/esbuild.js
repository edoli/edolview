const { build } = require("esbuild");

const baseConfig = {
  bundle: true,
  minify: process.env.NODE_ENV === "production",
  sourcemap: process.env.NODE_ENV !== "production",
};

const extensionConfig = {
  ...baseConfig,
  platform: "node",
  mainFields: ["module", "main"],
  format: "cjs",
  entryPoints: ["./src/extension.ts"],
  outfile: "./out/extension.js",
  external: ["vscode"],
};

const watchConfig = {
    watch: {
      onRebuild(error, result) {
        console.log("[watch] build started");
        if (error) {
          error.errors.forEach(error =>
            console.error(`> ${error.location.file}:${error.location.line}:${error.location.column}: error: ${error.text}`)
          );
        } else {
          console.log("[watch] build finished");
        }
      },
    },
  };
  
  (async () => {
    const args = process.argv.slice(2);
    try {
      if (args.includes("--watch")) {
        // Build and watch extension
        console.log("[watch] build started");
        await build({
          ...extensionConfig,
          ...watchConfig,
        });
        console.log("[watch] build finished");
      } else {
        // Build extension
        await build(extensionConfig);
        console.log("build complete");
      }
    } catch (err) {
      process.stderr.write(err.stderr);
      process.exit(1);
    }
  })();