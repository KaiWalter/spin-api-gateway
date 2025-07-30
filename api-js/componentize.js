import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

import { componentize } from "@bytecodealliance/componentize-js";

// AoT compilation makes use of weval (https://github.com/bytecodealliance/weval)
const enableAot = process.env.ENABLE_AOT == "1";

const jsSource = await readFile("index.js", "utf8");

const { component } = await componentize(jsSource, {
  witPath: resolve("../wit/shared-api.wit"),
  worldName: "api",
  enableAot,
});

await writeFile(
  "../target/wasm32-wasip2/debug/api-js.component.wasm",
  component,
);
