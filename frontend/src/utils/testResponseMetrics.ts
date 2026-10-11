import { calculateTestSummary, calculateChartPoints } from './responseMetrics.ts';
import type { AssertionResult } from '../components/ResponseViewer.svelte';

console.log('--- RUNNING RESPONSE METRICS UNIT TEST SUITE ---');

// 1. Empty assertions
const emptySummary = calculateTestSummary([]);
console.assert(emptySummary.total === 0, 'Empty total mismatch');
console.assert(emptySummary.passed === 0, 'Empty passed mismatch');
console.assert(emptySummary.failed === 0, 'Empty failed mismatch');
console.assert(emptySummary.passRate === 100, 'Empty passRate should default to 100');
console.log('✓ Empty test summary calculation');

// 2. Mixed assertion results
const mixedAssertions: AssertionResult[] = [
  { name: 'Status is 200', passed: true, duration_ms: 5 },
  { name: 'Response has user object', passed: true, duration_ms: 10 },
  { name: 'User email is valid', passed: false, error: 'Expected valid email, got null', duration_ms: 2 }
];

const mixedSummary = calculateTestSummary(mixedAssertions);
console.assert(mixedSummary.total === 3, `Expected 3 total, got ${mixedSummary.total}`);
console.assert(mixedSummary.passed === 2, `Expected 2 passed, got ${mixedSummary.passed}`);
console.assert(mixedSummary.failed === 1, `Expected 1 failed, got ${mixedSummary.failed}`);
console.assert(mixedSummary.passRate === 67, `Expected 67% passRate, got ${mixedSummary.passRate}`);
console.log('✓ Mixed assertions test summary calculation');

// 3. Chart points calculation from JSON array
const chartSource = [
  { category: 'Electronics', sales: 150 },
  { category: 'Books', sales: 75 },
  { category: 'Home', sales: 220 }
];
const points = calculateChartPoints(chartSource);
console.assert(points.length === 3, `Expected 3 points, got ${points.length}`);
console.assert(points[0].label === 'Electronics', 'Label 0 mismatch');
console.assert(points[0].value === 150, 'Value 0 mismatch');
console.assert(points[2].heightPct === 100, 'Max value should be 100% height');
console.log('✓ Chart points calculation');

console.log('--- ALL RESPONSE METRICS TESTS PASSED ---');
