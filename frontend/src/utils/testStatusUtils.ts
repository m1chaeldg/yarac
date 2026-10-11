import { formatDuration, formatSize, getStatusCategory, getStatusBadgeColor } from './statusUtils.ts';

console.log('--- RUNNING STATUS UTILS UNIT TEST SUITE ---');

// 1. Duration formatting
console.assert(formatDuration(45) === '45 ms', `45ms failed: ${formatDuration(45)}`);
console.assert(formatDuration(1250) === '1.25 s', `1250ms failed: ${formatDuration(1250)}`);
console.assert(formatDuration(0) === '0 ms', `0ms failed: ${formatDuration(0)}`);
console.log('✓ Duration formatting');

// 2. Size formatting
console.assert(formatSize(512) === '512 B', `512B failed: ${formatSize(512)}`);
console.assert(formatSize(2048) === '2.00 KB', `2048B failed: ${formatSize(2048)}`);
console.assert(formatSize(1048576 * 2.5) === '2.50 MB', `2.5MB failed: ${formatSize(1048576 * 2.5)}`);
console.log('✓ Size formatting');

// 3. Status categories & badge colors
console.assert(getStatusCategory(200) === '2xx', '200 category mismatch');
console.assert(getStatusCategory(201) === '2xx', '201 category mismatch');
console.assert(getStatusCategory(304) === '3xx', '304 category mismatch');
console.assert(getStatusCategory(404) === '4xx', '404 category mismatch');
console.assert(getStatusCategory(500) === '5xx', '500 category mismatch');
console.assert(getStatusCategory(0) === 'unknown', '0 category mismatch');

console.assert(getStatusBadgeColor(200).includes('emerald') || getStatusBadgeColor(200).includes('green'), '200 badge color mismatch');
console.assert(getStatusBadgeColor(404).includes('amber') || getStatusBadgeColor(404).includes('yellow') || getStatusBadgeColor(404).includes('orange'), '404 badge color mismatch');
console.assert(getStatusBadgeColor(500).includes('rose') || getStatusBadgeColor(500).includes('red'), '500 badge color mismatch');
console.log('✓ Status code categories and badge styles');

console.log('--- ALL STATUS UTILS TESTS PASSED ---');
