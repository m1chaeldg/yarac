<script lang="ts">
  import { workspaceStore, type YaracRequest } from '../../stores/workspace.svelte';

  interface Props {
    req: YaracRequest;
  }

  let { req }: Props = $props();

  function ensureSettings() {
    if (!req.settings) {
      req.settings = { inheritAuth: true, verifySsl: true, followRedirects: true, maxRedirects: 5, timeoutMs: 0 };
    }
  }
</script>

<div class="max-w-2xl space-y-4 font-sans">
  <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block">
    Request Execution Settings
  </span>

  <div class="p-4 bg-slate-900/60 border border-slate-800 rounded-lg space-y-4">
    <!-- Follow Redirects -->
    <div class="flex items-center justify-between">
      <div>
        <label for="set-follow-redirect" class="text-xs font-semibold text-slate-200 block cursor-pointer">Follow Redirects</label>
        <span class="text-[11px] text-slate-500">Automatically follow HTTP 3xx redirection responses</span>
      </div>
      <input
        id="set-follow-redirect"
        type="checkbox"
        checked={req.settings?.followRedirects ?? true}
        onchange={(e) => {
          ensureSettings();
          if (req.settings) {
            req.settings.followRedirects = (e.target as HTMLInputElement).checked;
            workspaceStore.updateActiveRequest({ settings: req.settings });
          }
        }}
        class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0 cursor-pointer"
      />
    </div>

    <div class="border-t border-slate-800/80"></div>

    <!-- Max Redirects -->
    <div class="flex items-center justify-between">
      <div>
        <label for="set-max-redirects" class="text-xs font-semibold text-slate-200 block">Max Redirects</label>
        <span class="text-[11px] text-slate-500">Maximum number of consecutive redirects before aborting</span>
      </div>
      <input
        id="set-max-redirects"
        type="number"
        min="0"
        max="50"
        value={req.settings?.maxRedirects ?? 5}
        oninput={(e) => {
          ensureSettings();
          if (req.settings) {
            req.settings.maxRedirects = parseInt((e.target as HTMLInputElement).value || '5', 10);
            workspaceStore.updateActiveRequest({ settings: req.settings });
          }
        }}
        class="w-24 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500 text-right"
      />
    </div>

    <div class="border-t border-slate-800/80"></div>

    <!-- Timeout (ms) -->
    <div class="flex items-center justify-between">
      <div>
        <label for="set-timeout" class="text-xs font-semibold text-slate-200 block">Timeout (ms)</label>
        <span class="text-[11px] text-slate-500">Maximum execution time before request cancels. <strong>0</strong> = indefinite (no timeout)</span>
      </div>
      <div class="flex items-center gap-2">
        <input
          id="set-timeout"
          type="number"
          min="0"
          step="500"
          value={req.settings?.timeoutMs ?? 0}
          oninput={(e) => {
            ensureSettings();
            if (req.settings) {
              req.settings.timeoutMs = parseInt((e.target as HTMLInputElement).value || '0', 10);
              workspaceStore.updateActiveRequest({ settings: req.settings });
            }
          }}
          class="w-28 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:border-sky-500 text-right"
        />
        <span class="text-[11px] text-slate-400 font-mono">ms</span>
      </div>
    </div>

    <div class="border-t border-slate-800/80"></div>

    <!-- Verify SSL -->
    <div class="flex items-center justify-between">
      <div>
        <label for="set-verify-ssl" class="text-xs font-semibold text-slate-200 block cursor-pointer">Verify SSL Certificates</label>
        <span class="text-[11px] text-slate-500">Enforce valid TLS / HTTPS certificates for remote server</span>
      </div>
      <input
        id="set-verify-ssl"
        type="checkbox"
        checked={req.settings?.verifySsl ?? true}
        onchange={(e) => {
          ensureSettings();
          if (req.settings) {
            req.settings.verifySsl = (e.target as HTMLInputElement).checked;
            workspaceStore.updateActiveRequest({ settings: req.settings });
          }
        }}
        class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0 cursor-pointer"
      />
    </div>

    <div class="border-t border-slate-800/80"></div>

    <!-- Inherit Auth -->
    <div class="flex items-center justify-between">
      <div>
        <label for="set-inherit-auth" class="text-xs font-semibold text-slate-200 block cursor-pointer">Inherit Collection Auth</label>
        <span class="text-[11px] text-slate-500">Automatically attach parent collection authentication credentials</span>
      </div>
      <input
        id="set-inherit-auth"
        type="checkbox"
        checked={req.settings?.inheritAuth ?? (req.auth.type === 'inherit')}
        onchange={(e) => {
          ensureSettings();
          if (req.settings) {
            req.settings.inheritAuth = (e.target as HTMLInputElement).checked;
            workspaceStore.updateActiveRequest({ settings: req.settings });
          }
        }}
        class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0 cursor-pointer"
      />
    </div>
  </div>
</div>
