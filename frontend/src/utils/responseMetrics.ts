/**
 * Response metrics, assertion summaries, and chart helpers.
 */

export interface TestSummary {
  total: number;
  passed: number;
  failed: number;
  passRate: number;
}

export interface ChartPoint {
  label: string;
  value: number;
  heightPct?: number;
}

export function calculateTestSummary(assertions: Array<{ passed: boolean }> | undefined): TestSummary {
  const list = assertions || [];
  const total = list.length;
  const passed = list.filter(a => a.passed).length;
  const failed = total - passed;
  const passRate = total > 0 ? Math.round((passed / total) * 100) : 100;
  return { total, passed, failed, passRate };
}

export function calculateChartPoints(data: any): ChartPoint[] {
  if (!Array.isArray(data) || data.length === 0) return [];
  const points: ChartPoint[] = [];

  for (const item of data) {
    if (typeof item === 'object' && item !== null) {
      let label = '';
      let value = 0;
      for (const [k, v] of Object.entries(item)) {
        if (!label && typeof v === 'string') {
          label = v;
        } else if (!value && typeof v === 'number') {
          value = v;
        }
      }
      if (label && value) {
        points.push({ label, value });
      }
    }
  }

  const maxVal = Math.max(...points.map(p => p.value), 1);
  return points.map(p => ({
    ...p,
    heightPct: Math.round((p.value / maxVal) * 100)
  }));
}
