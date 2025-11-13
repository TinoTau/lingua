import { DefaultWasmModuleLoader, createBridgeRuntimeFactory } from "../../../core/bindings/typescript";
import { createCoreEngineBridgeFactory } from "./engineBridgeFactory";
import { BackgroundCommandRouter } from "./commandRouter";
import { ConsoleLogger } from "./logger";
import { ChannelHub, ChannelName } from "./channelHub";
import { EngineEvent } from "../shared/coreTypes";

const logger = new ConsoleLogger();
const wasmLoader = new DefaultWasmModuleLoader();
const runtimeFactory = createBridgeRuntimeFactory(wasmLoader);
const bridgeFactory = createCoreEngineBridgeFactory({
  runtimeFactory,
  logger,
});
const channelHub = new ChannelHub();

const router = new BackgroundCommandRouter({
  bridgeFactory,
  bridgeOptions: {
    wasmModulePath: chrome.runtime.getURL("core/engine.wasm"),
  },
  logger,
  onEvent: (event: EngineEvent) => {
    channelHub.broadcastAll(event);
  },
});

chrome.runtime.onConnect.addListener((port) => {
  const channel = port.name as ChannelName;
  if (channel !== "content" && channel !== "popup" && channel !== "options") {
    logger.warn("unknown channel connection", { channel });
    return;
  }
  channelHub.register(channel, port);
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

chrome.runtime.onSuspend?.addListener(() => {
  router.handle({ type: "engine/shutdown", payload: undefined }).catch((error) => {
    logger.error("failed to shutdown engine on suspend", { error });
  });
  channelHub.close();
});