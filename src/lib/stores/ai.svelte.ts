import { ai, aiEvents, type CloudStatus, type JobStatus, type ModelStatus, type Overview, type Task, type Transcript } from '$lib/ipc/ai';
import { isReady, languageModelId, speechModelId } from '$lib/ai';
import { t, type MessageKey } from '$lib/i18n/index.svelte';
import { toast } from './toasts.svelte';

/** FrameLy AI state shared by the Transcriptions page and Settings: hardware, models,
 * providers, transcripts and live jobs. Everything reflects what the backend reports. */
class AiStore {
  overview = $state<Overview | null>(null);
  transcripts = $state<Transcript[]>([]);
  jobs = $state<Record<number, JobStatus>>({});
  /** Bumped when a transcript changes (new block, edit, generated content). */
  changed = $state<{ id: number; n: number }>({ id: 0, n: 0 });
  private started = false;
  /** FrameLy Cloud connection (null while checking). */
  cloud = $state<CloudStatus | null>(null);
  cloudChecking = $state(false);
  private cloudCheckedAt = 0;

  async init(): Promise<void> {
    if (this.started) {
      void this.refresh();
      return;
    }
    this.started = true;
    await aiEvents.onModels((m) => this.patchModel(m));
    await aiEvents.onJob((j) => this.onJob(j));
    await aiEvents.onTranscript((id) => {
      this.changed = { id, n: this.changed.n + 1 };
      void this.loadList();
    });
    await aiEvents.onNotice((n) => {
      if (n.code === 'auto_skipped') toast('info', t('ai.notice.autoSkipped', { task: t(`ai.task.${n.task}` as MessageKey) }), 5000);
    });
    for (const j of await ai.jobs()) this.jobs[j.id] = j;
    await this.refresh();
  }

  async refresh(): Promise<void> {
    await Promise.all([this.loadOverview(), this.loadList()]);
  }

  async loadOverview(): Promise<void> {
    try {
      this.overview = await ai.overview();
    } catch (e) {
      console.error('ai_overview failed', e);
    }
  }

  async loadList(): Promise<void> {
    try {
      this.transcripts = await ai.list();
    } catch (e) {
      console.error('transcripts_list failed', e);
    }
  }

  private patchModel(m: ModelStatus): void {
    if (!this.overview) return;
    const i = this.overview.models.findIndex((x) => x.id === m.id);
    if (i >= 0) this.overview.models[i] = m;
    // Installing or removing a model changes provider availability and storage.
    if (m.state === 'installed' || m.state === 'missing' || m.state === 'corrupt') void this.loadOverview();
    if (m.error && m.state !== 'downloading') toast('error', t(`error.${m.error}` as MessageKey) ?? t('error.generic'), 5000);
  }

  private onJob(j: JobStatus): void {
    this.jobs[j.id] = j;
    if (j.state === 'failed' && j.error && j.error.code !== 'cancelled') {
      toast('error', t(`error.${j.error.code}` as MessageKey) ?? t('error.generic'), 6000);
    }
    if (j.state === 'done' || j.state === 'failed' || j.state === 'cancelled') {
      void this.loadList();
      void this.loadOverview();
    }
  }

  model(id: string): ModelStatus | undefined {
    return this.overview?.models.find((m) => m.id === id);
  }

  get speechModel(): ModelStatus | undefined {
    return this.overview ? this.model(speechModelId(this.overview.defaultPrecision)) : undefined;
  }

  get languageModel(): ModelStatus | undefined {
    return this.overview ? this.model(languageModelId(this.overview.profile)) : undefined;
  }

  /** FrameLy AI can transcribe with the default precision. */
  get canTranscribe(): boolean {
    return isReady(this.speechModel);
  }

  /** The active (queued or running) job of a kind for a transcript. */
  active(transcriptId: number, kind: 'transcribe' | Task): JobStatus | undefined {
    return Object.values(this.jobs)
      .filter((j) => j.transcriptId === transcriptId && j.kind === kind && (j.state === 'queued' || j.state === 'running'))
      .sort((a, b) => b.id - a.id)[0];
  }

  /** Checks FrameLy Cloud (no content sent); results are reused for a minute. */
  async checkCloud(force = false): Promise<void> {
    if (this.cloudChecking || (!force && Date.now() - this.cloudCheckedAt < 60_000 && this.cloud)) return;
    this.cloudChecking = true;
    try {
      this.cloud = await ai.cloudStatus();
      this.cloudCheckedAt = Date.now();
    } catch {
      this.cloud = { state: 'offline', dataUse: null, maxInputChars: null, requestsPerDay: null };
    } finally {
      this.cloudChecking = false;
    }
  }

  /** What is happening to a transcript right now: the running job, else the next queued one. */
  current(transcriptId: number): JobStatus | undefined {
    const live = Object.values(this.jobs).filter((j) => j.transcriptId === transcriptId && (j.state === 'queued' || j.state === 'running'));
    return live.find((j) => j.state === 'running') ?? live.sort((a, b) => a.id - b.id)[0];
  }

  async modelAction(id: string, action: Parameters<typeof ai.modelAction>[1]): Promise<void> {
    try {
      this.patchModel(await ai.modelAction(id, action));
    } catch (e) {
      const code = (e as { code?: string }).code;
      toast('error', (code && t(`error.${code}` as MessageKey)) || t('error.generic'), 5000);
    }
  }
}

export const aiStore = new AiStore();
