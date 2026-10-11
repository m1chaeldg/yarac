/**
 * Status, time, and size formatting utilities for HTTP response viewers.
 */

export function formatDuration(durationMs: number): string {
  if (!durationMs || durationMs < 0) return '0 ms';
  if (durationMs < 1000) {
    return `${Math.round(durationMs)} ms`;
  }
  return `${(durationMs / 1000).toFixed(2)} s`;
}

export function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0) return '0 B';
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(2)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function getStatusCategory(status: number): '2xx' | '3xx' | '4xx' | '5xx' | 'unknown' {
  if (status >= 200 && status < 300) return '2xx';
  if (status >= 300 && status < 400) return '3xx';
  if (status >= 400 && status < 500) return '4xx';
  if (status >= 500 && status < 600) return '5xx';
  return 'unknown';
}

export function getStatusBadgeColor(status: number): string {
  const cat = getStatusCategory(status);
  switch (cat) {
    case '2xx':
      return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30';
    case '3xx':
      return 'text-sky-400 bg-sky-500/10 border-sky-500/30';
    case '4xx':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/30';
    case '5xx':
      return 'text-rose-400 bg-rose-500/10 border-rose-500/30';
    default:
      return 'text-slate-400 bg-slate-500/10 border-slate-500/30';
  }
}
