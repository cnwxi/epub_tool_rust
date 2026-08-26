export type TaskType =
  | "reformat_epub"
  | "decrypt_epub"
  | "encrypt_epub"
  | "encrypt_font"
  | "decrypt_font"
  | "webp_to_img"
  | "image_compress"
  | "image_to_webp"
  | "chinese_convert"
  | "replace_cover";
export type SectionKey = TaskType | "overview" | "engine" | "settings" | "about";
export type FontLoadStatus = "idle" | "loading" | "loaded" | "error";
export type OcrCharPolicy = "strict" | "compatible";
export type EngineState =
  | "starting"
  | "ready"
  | "busy"
  | "unavailable";
export type TaskOutputDirectoryMap = Record<TaskType, string>;

export interface QueuedFile {
  path: string;
  name: string;
  fontFamilies: string[];
  selectedFontFamilies: string[];
  fontLoadStatus: FontLoadStatus;
  fontLoadError: string;
  coverPath: string;
  coverPreviewUrl: string;
}

export interface ImagePreviewResponse {
  bytes: number[];
  mimeType: string;
}

export interface NewTaskSettings {
  jpegQuality: number;
  webpQuality: number;
  pngToJpg: boolean;
  imageCompressQuantizePng: boolean;
  webpToImageQuality: number;
  webpToImageQuantizePng: boolean;
  imageWebpQuality: number;
  chineseDirection: "s2t" | "t2s";
}

export type FontTargetResult = Required<FontTargetResultJson>;

export type FontTargetProgressEvent = Required<FontScanProgressJson> & {
  result: FontTargetResult;
};

export type TaskRequest = EngineRequestJson;
type EngineError = Required<EngineErrorJson>;
type FontScanResult = Required<Omit<FontScanResultJson, "results">> & {
  results: FontTargetResult[];
};

type FileIssue = Required<FileIssueJson>;
type TaskSummary = Required<TaskSummaryJson>;

export type TaskResult = Omit<Required<TaskResultJson>, "errors" | "skipped" | "summary"> & {
  errors: FileIssue[];
  skipped: FileIssue[];
  summary: TaskSummary;
};

const normalizeTaskCount = (value: unknown): number =>
  typeof value === "number" && Number.isFinite(value) && value >= 0 ? Math.floor(value) : 0;

const normalizeFileIssues = (value: unknown): FileIssue[] =>
  Array.isArray(value)
    ? value.map((item) => {
      const raw = item && typeof item === "object" ? (item as Partial<FileIssue>) : {};
      return {
        inputFile: typeof raw.inputFile === "string" ? raw.inputFile : "",
        message: typeof raw.message === "string" ? raw.message : "",
      };
    })
    : [];

/**
 * Tauri IPC serializes Protobuf messages as JSON. Normalize omitted Protobuf
 * defaults so the UI never treats a partial wire payload as a complete result.
 */
export const normalizeTaskResult = (
  value: TaskResult | null | undefined,
): TaskResult | null => {
  if (!value || typeof value !== "object") {
    return null;
  }

  const raw = value as Partial<TaskResult>;
  const summary = (raw.summary ?? {}) as Partial<TaskSummary>;
  return {
    ok: raw.ok === true,
    status: typeof raw.status === "string" ? raw.status : "",
    outputs: Array.isArray(raw.outputs)
      ? raw.outputs.filter((output): output is string => typeof output === "string")
      : [],
    errors: normalizeFileIssues(raw.errors),
    skipped: normalizeFileIssues(raw.skipped),
    summary: {
      total: normalizeTaskCount(summary.total),
      success: normalizeTaskCount(summary.success),
      failed: normalizeTaskCount(summary.failed),
      skipped: normalizeTaskCount(summary.skipped),
    },
    logPath: typeof raw.logPath === "string" ? raw.logPath : "",
  };
};

export type TaskEvent = Omit<TaskEventJson, "progress" | "result"> & {
  event: string;
  taskId: string;
  status: string;
  progress: number;
  message: string;
  level: string;
  result?: TaskResult;
};

export type EngineEvent = Omit<Required<EngineEventJson>, "taskEvent" | "fontScanProgress"> & {
  taskEvent?: TaskEvent;
  fontScanProgress?: FontTargetProgressEvent;
};

export type EngineResponse = Omit<
  Required<EngineResponseJson>,
  "taskResult" | "fontScanResult" | "error"
> & {
  taskResult?: TaskResult;
  fontScanResult?: FontScanResult;
  error?: EngineError;
};

export interface AppSettings {
  autoOpenOutputFolder: boolean;
  autoOpenLogFile: boolean;
  autoCheckUpdates: boolean;
  keepHistoryCount: number;
}

export interface EngineStatus {
  state: EngineState;
  message: string;
  lastError?: string | null;
}

export interface PlatformCapabilities {
  platform: "linux" | "macos" | "windows" | "unknown";
  runtime: "inProcess" | "browser";
  supportsDirectoryPicker: boolean;
  supportsDirectoryScan: boolean;
  supportsOpenPath: boolean;
  requiresOutputExport: boolean;
  supportsFileAssociations: boolean;
  supportsFontOcr: boolean;
}

export interface FontDecryptSettings {
  ocrCharPolicy: OcrCharPolicy;
  minOcrConfidence: number;
}

export interface TaskAggregateStats {
  total: number;
  success: number;
  failed: number;
  skipped: number;
}

export interface UpdateCheckState {
  checkedAt: string;
  latestVersion: string;
  latestReleaseUrl: string;
  status: "idle" | "checking" | "available" | "latest" | "error";
  message: string;
}

export interface TaskHistoryEntry {
  id: string;
  createdAt: string;
  taskType: TaskType;
  status: string;
  summary: TaskResult["summary"];
  firstOutput?: string | null;
}
import type {
  EngineErrorJson,
  EngineEventJson,
  EngineRequestJson,
  EngineResponseJson,
  FileIssueJson,
  FontScanResultJson,
  FontScanProgressJson,
  FontTargetResultJson,
  TaskEventJson,
  TaskResultJson,
  TaskSummaryJson,
} from "./generated/epub_tool/v1/engine_pb";
