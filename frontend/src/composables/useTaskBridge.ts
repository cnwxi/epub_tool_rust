import { Channel, invoke } from "@tauri-apps/api/core";
import { shallowRef } from "vue";

import { normalizeTaskResult } from "../types";
import type {
  EngineEvent,
  EngineResponse,
  ImagePreviewResponse,
  PlatformCapabilities,
  EngineStatus,
  TaskEvent,
  TaskRequest,
} from "../types";

const isTauriRuntime = (): boolean =>
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

const normalizeEngineEvent = (event: EngineEvent): EngineEvent => {
  if (!event.taskEvent) {
    return event;
  }
  const taskEvent: TaskEvent = {
    ...event.taskEvent,
    result: normalizeTaskResult(event.taskEvent.result) ?? undefined,
  };
  return { ...event, taskEvent };
};

const normalizeEngineResponse = (response: EngineResponse): EngineResponse => ({
  ...response,
  taskResult: normalizeTaskResult(response.taskResult) ?? undefined,
});

export function useTaskBridge() {
  const isMobileFrontend = import.meta.env.MODE === "mobile";
  const platformCapabilities = shallowRef<PlatformCapabilities>({
    platform: isMobileFrontend ? "android" : "unknown",
    runtime: "browser",
    supportsDirectoryPicker: false,
    supportsDirectoryScan: false,
    supportsOpenPath: false,
    requiresOutputExport: isMobileFrontend,
    supportsFileAssociations: false,
    supportsFontOcr: !isMobileFrontend,
  });

  const refreshPlatformCapabilities = async (): Promise<PlatformCapabilities> => {
    if (isTauriRuntime()) {
      platformCapabilities.value = await invoke<PlatformCapabilities>(
        "get_platform_capabilities",
      );
    }
    return platformCapabilities.value;
  };

  const runTask = async (
    request: TaskRequest,
    onEvent: (event: EngineEvent) => void,
  ): Promise<EngineResponse> => {
    if (!isTauriRuntime()) {
      throw new Error("当前环境不支持该功能，请在应用中使用。");
    }

    const channel = new Channel<EngineEvent>((event) => {
      onEvent(normalizeEngineEvent(event));
    });

    const response = await invoke<EngineResponse>("run_epub_task", {
      request,
      onEvent: channel,
    });
    return normalizeEngineResponse(response);
  };

  const listFontTargetsBatch = async (
    filePaths: string[],
    onEvent: (event: EngineEvent) => void,
  ): Promise<EngineResponse> => {
    if (!isTauriRuntime()) {
      throw new Error("当前环境不支持该功能，请在应用中使用。");
    }
    const channel = new Channel<EngineEvent>((event) => {
      onEvent(normalizeEngineEvent(event));
    });
    const response = await invoke<EngineResponse>("list_font_targets_batch", {
      request: {
        protocolVersion: "PROTOCOL_VERSION_V1",
        requestId: crypto.randomUUID(),
        scanFonts: { inputFiles: filePaths },
      },
      onEvent: channel,
    });
    return normalizeEngineResponse(response);
  };

  const collectEpubFiles = async (directoryPath: string): Promise<string[]> => {
    if (!isTauriRuntime()) {
      return [];
    }
    return invoke<string[]>("collect_epub_files", {
      directoryPath,
    });
  };

  const getLogPath = async (): Promise<string> => {
    if (!isTauriRuntime()) {
      return "";
    }
    return invoke<string>("get_log_path");
  };

  const getPersistedStorePath = async (): Promise<string> => {
    if (!isTauriRuntime()) {
      return "";
    }
    return invoke<string>("get_persisted_store_path");
  };

  const getEngineStatus = async (): Promise<EngineStatus | null> => {
    if (!isTauriRuntime()) {
      return null;
    }
    return invoke<EngineStatus>("get_engine_status");
  };

  const loadPersistedState = async <T>(
    key: string,
  ): Promise<{ found: boolean; value: T | null }> => {
    if (!isTauriRuntime()) {
      return { found: false, value: null };
    }
    return invoke<{ found: boolean; value: T | null }>("load_persisted_state", {
      key,
    });
  };

  const resolveInputSources = async (inputPaths: string[]): Promise<string[]> => {
    if (!isTauriRuntime()) {
      return inputPaths;
    }
    return invoke<string[]>("resolve_input_sources", {
      inputPaths,
    });
  };

  const validateOutputDirectory = async (directoryPath: string): Promise<void> => {
    if (!isTauriRuntime()) {
      return;
    }
    await invoke("validate_output_directory", { directoryPath });
  };

  const openPath = async (path: string): Promise<void> => {
    if (!isTauriRuntime()) {
      return;
    }
    await invoke("open_path", { path });
  };

  const readImagePreview = async (path: string): Promise<ImagePreviewResponse> => {
    if (!isTauriRuntime()) {
      throw new Error("当前环境不支持本地图片预览。");
    }
    return invoke<ImagePreviewResponse>("read_image_preview", { path });
  };

  const savePersistedState = async (key: string, value: unknown): Promise<void> => {
    if (!isTauriRuntime()) {
      return;
    }
    await invoke("save_persisted_state", { key, value });
  };

  const stageSourceForTask = async (sourcePath: string, extension: string): Promise<string> => {
    if (!isTauriRuntime()) return sourcePath;
    return invoke<string>("stage_source_for_task", { sourcePath, extension });
  };
  const exportOutput = async (sourcePath: string, destinationPath: string): Promise<void> => {
    if (!isTauriRuntime()) return;
    await invoke("export_output", { sourcePath, destinationPath });
  };
  const exportLog = async (destinationPath: string): Promise<void> => {
    if (!isTauriRuntime()) return;
    await invoke("export_log", { destinationPath });
  };
  const takeOpenedSources = async (): Promise<string[]> => {
    if (!isTauriRuntime()) return [];
    return invoke<string[]>("take_opened_sources");
  };

  return {
    collectEpubFiles,
    exportLog,
    exportOutput,
    getLogPath,
    getPersistedStorePath,
    getEngineStatus,
    isTauriRuntime,
    listFontTargetsBatch,
    loadPersistedState,
    openPath,
    readImagePreview,
    refreshPlatformCapabilities,
    resolveInputSources,
    runTask,
    savePersistedState,
    stageSourceForTask,
    takeOpenedSources,
    platformCapabilities,
    validateOutputDirectory,
  };
}
