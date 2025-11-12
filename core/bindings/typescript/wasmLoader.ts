import { WasmEngineModule, WasmModuleLoader } from "./runtime";

export class DefaultWasmModuleLoader implements WasmModuleLoader {
  async load(path: string): Promise<WasmEngineModule> {
    const module = (await import(/* webpackIgnore: true */ path)) as unknown as WasmEngineModule;
    if (!module) {
      throw new Error(`Failed to load wasm module from ${path}`);
    }
    return module;
  }
}

