import { DefaultWasmModuleLoader, createBridgeRuntimeFactory } from "../../../core/bindings/typescript";
import { createCoreEngineBridgeFactory } from "./engineBridgeFactory";
import { BackgroundCommandRouter } from "./commandRouter";
import { ConsoleLogger } from "./logger";

const logger = new ConsoleLogger();
const wasmLoader = new DefaultWasmModuleLoader();
const runtimeFactory = createBridgeRuntimeFactory(wasmLoader);
const bridgeFactory = createCoreEngineBridgeFactory({
  runtimeFactory,
  logger,
});

const router = new BackgroundCommandRouter({
  bridgeFactory,
  bridgeOptions: {
    wasmModulePath: chrome.runtime.getURL("core/engine.wasm"),
  },
  logger,
});

chrome.runtime.onMessage.addListener((command, _sender, sendResponse) => {
  void router
    .handle(command)
    .then((response) => sendResponse(response))
    .catch((error: unknown) => {
      logger.error("command handling failed", { error });
      sendResponse({ ok: false, error: error instanceof Error ? error.message : String(error) });
    });
  return true;
});

