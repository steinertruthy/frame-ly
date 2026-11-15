import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Settings } from './types';

export type AiProfile = 'auto' | 'basic' | 'balanced' | 'advanced';
export type Precision = 'auto' | 'fast' | 'balanced' | 'max';
export type Detail = 'brief' | 'balanced' | 'detailed';
export type ProviderId = 'framely' | 'apple' | 'anthropic' | 'openai';
export type Task = 'summary' | 'topics';

export interface Hardware {
  os: string;
  cpu: string;
  cores: number;
  ramBytes: number;
  gpu: string | null;
  vramBytes: number | null;
  accel: 'metal' | 'cpu';
  avx2: boolean;
  supported: boolean;
  recommended: Exclude<AiProfile, 'auto'>;
}

export type ModelState = 'missing' | 'downloading' | 'verifying' | 'paused' | 'installed' | 'update' | 'corrupt';

export interface ModelStatus {
  id: string;
  kind: 'speech' | 'language';
  version: number;
  file: string;
  size: number;
  license: string;
  ramMb: number;
  context: number;
  state: ModelState;
  received: number;
  installedVersion: number | null;
  diskBytes: number;
  error: string | null;
  inUse: boolean;
}

export interface ProviderInfo {
  id: ProviderId;
  local: boolean;
  needsInternet: boolean;
  needsKey: boolean;
  keyConfigured: boolean;
  available: boolean;
  reason: string | null;
  contextTokens: number;
  paid: boolean;
  model: string;
  consented: boolean;
}

export interface Overview {
  hardware: Hardware;
  profile: Exclude<AiProfile, 'auto'>;
  defaultPrecision: Exclude<Precision, 'auto'>;
  models: ModelStatus[];
  providers: ProviderInfo[];
  storage: { modelsBytes: number; tempBytes: number; freeBytes: number | null };
  importable: string[];
  anthropicModels: string[];
}

export interface MediaRef {
  path: string;
  name: string;
  kind: 'video' | 'audio';
  size: number;
  mtime: number;
  durationMs: number;
}

export type TranscriptStatus = 'queued' | 'running' | 'interrupted' | 'cancelled' | 'failed' | 'done';

export interface Transcript {
  id: number;
  media: MediaRef;
  language: string;
  detectedLanguage: string | null;
  precision: Exclude<Precision, 'auto'>;
  modelId: string;
  status: TranscriptStatus;
  doneMs: number;
  blocksDone: number;
  error: string | null;
  createdAt: number;
  updatedAt: number;
  hasSummary: boolean;
  hasTopics: boolean;
  edited: boolean;
}

export interface Word {
  text: string;
  startMs: number;
  endMs: number;
  p: number;
}

export interface Segment {
  idx: number;
  startMs: number;
  endMs: number;
  text: string;
  words: Word[];
  block: number;
  edited: boolean;
}

export interface Paragraph {
  startMs: number;
  endMs: number;
  segments: number[];
}

export interface SummaryContent {
  overview: string;
  points: { text: string; ms: number | null }[];
}

export interface TopicNode {
  title: string;
  ms: number | null;
  points: string[];
  children: TopicNode[];
}

export interface TimedItem {
  text: string;
  ms: number | null;
}

export interface TopicsContent {
  topics: TopicNode[];
  decisions: TimedItem[];
  actions: (TimedItem & { owner: string; due: string })[];
  dates: TimedItem[];
  numbers: TimedItem[];
  keywords: string[];
}

export interface Generated<T> {
  detail: Detail;
  provider: ProviderId;
  model: string;
  content: T;
  createdAt: number;
  edited: boolean;
}

export interface Detailed {
  transcript: Transcript;
  segments: Segment[];
  paragraphs: Paragraph[];
  summary: Generated<SummaryContent> | null;
  topics: Generated<TopicsContent> | null;
  mediaAvailable: boolean;
}

export interface TimeEstimate {
  ms: number;
  approximate: boolean;
  runs: number;
}

export interface Estimate {
  media: MediaRef;
  precision: Exclude<Precision, 'auto'>;
  model: ModelStatus;
  time: TimeEstimate;
  accel: 'metal' | 'cpu';
  existing: Transcript | null;
}

export interface GenerationEstimate {
  time: TimeEstimate | null;
  inputTokens: number;
  costUsd: [number, number] | null;
}

export type JobPhase = 'queued' | 'preparing' | 'loading' | 'transcribing' | 'generating' | 'combining' | 'saving';

export interface JobStatus {
  id: number;
  transcriptId: number;
  kind: 'transcribe' | Task;
  state: 'queued' | 'running' | 'done' | 'failed' | 'cancelled';
  phase: JobPhase;
  progress: number | null;
  etaMs: number | null;
  part: [number, number] | null;
  provider: ProviderId | null;
  error: { code: string; message: string } | null;
}

export interface ExportLabels {
  decisions: string;
  actions: string;
  dates: string;
  numbers: string;
  owner: string;
  due: string;
}

export type ExportWhat = 'transcript' | 'summary' | 'topics';
export type ExportFormat = 'txt' | 'srt' | 'vtt' | 'md' | 'pdf';

export const ai = {
  overview: () => invoke<Overview>('ai_overview'),
  modelAction: (id: string, action: 'download' | 'pause' | 'cancel' | 'remove' | 'verify' | 'repair') =>
    invoke<ModelStatus>('ai_model_action', { id, action }),
  estimate: (path: string, precision: Precision) => invoke<Estimate>('ai_estimate', { path, precision }),
  generationEstimate: (id: number, task: Task, provider: ProviderId) =>
    invoke<GenerationEstimate>('ai_generation_estimate', { id, task, provider }),
  transcribe: (path: string, precision: Precision, language: string) =>
    invoke<{ transcript: Transcript; jobId: number | null }>('transcription_start', { path, precision, language }),
  list: () => invoke<Transcript[]>('transcripts_list'),
  get: (id: number) => invoke<Detailed>('transcript_get', { id }),
  find: (path: string) => invoke<Transcript | null>('transcript_find', { path }),
  editSegment: (id: number, idx: number, text: string) => invoke<void>('transcript_edit', { id, idx, text }),
  remove: (id: number) => invoke<void>('transcript_delete', { id }),
  saveGenerated: (id: number, kind: Task, content: unknown) => invoke<void>('generated_save', { id, kind, content }),
  generate: (id: number, task: Task, provider: ProviderId, detail: Detail) =>
    invoke<number>('generation_start', { id, task, provider, detail }),
  cancelJob: (jobId: number) => invoke<void>('ai_job_cancel', { jobId }),
  jobs: () => invoke<JobStatus[]>('ai_jobs'),
  export: (request: { id: number; what: ExportWhat; format: ExportFormat; path: string; title: string; meta: string; labels: ExportLabels }) =>
    invoke<void>('transcript_export', { request }),
  setKey: (provider: ProviderId, key: string) => invoke<void>('provider_key_set', { provider, key }),
  removeProvider: (provider: ProviderId) => invoke<Settings>('provider_remove', { provider }),
  testProvider: (provider: ProviderId) => invoke<string[]>('provider_test', { provider }),
  consent: (provider: ProviderId, granted: boolean) => invoke<Settings>('ai_consent', { provider, granted }),
  clearTemp: () => invoke<number>('ai_clear_temp'),
  probe: (path: string) => invoke<MediaRef>('media_probe', { path }),
};

export const aiEvents = {
  onJob: (cb: (s: JobStatus) => void): Promise<UnlistenFn> => listen<JobStatus>('ai://job', (e) => cb(e.payload)),
  onModels: (cb: (s: ModelStatus) => void): Promise<UnlistenFn> => listen<ModelStatus>('ai://models', (e) => cb(e.payload)),
  onTranscript: (cb: (id: number) => void): Promise<UnlistenFn> => listen<number>('ai://transcript', (e) => cb(e.payload)),
  onNotice: (cb: (n: { code: string; task: Task; provider: ProviderId; reason: string }) => void): Promise<UnlistenFn> =>
    listen('ai://notice', (e) => cb(e.payload as { code: string; task: Task; provider: ProviderId; reason: string })),
};
