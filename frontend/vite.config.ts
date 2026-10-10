import { defineConfig, Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import child_process from 'node:child_process';

function yaracWorkspacePlugin(): Plugin {
  const workspaceRoot = path.resolve(__dirname, '../workspace');

  // Ensure workspace directory exists
  if (!fs.existsSync(workspaceRoot)) {
    fs.mkdirSync(workspaceRoot, { recursive: true });
  }

  // Active SSE clients for file watcher
  const clients = new Set<any>();

  // Watch workspace directory for external changes
  let fsWatcher: fs.FSWatcher | null = null;
  const selfWriteTimes = new Map<string, number>();

  function setupWatcher() {
    try {
      fsWatcher = fs.watch(workspaceRoot, { recursive: true }, (eventType, filename) => {
        if (!filename) return;
        const normalized = filename.replace(/\\/g, '/');
        if (!normalized.endsWith('.yarac') && normalized !== 'workspace.json') return;

        const fullPath = path.join(workspaceRoot, filename);

        // Echo suppression: ignore writes originating from Yarac within 1200ms
        const expiry = selfWriteTimes.get(fullPath) || 0;
        if (Date.now() < expiry) {
          return;
        }

        let content: string | undefined;
        try {
          if (fs.existsSync(fullPath) && fs.statSync(fullPath).isFile()) {
            content = fs.readFileSync(fullPath, 'utf8');
          }
        } catch {
          // File may have been deleted
        }

        const payload = JSON.stringify({
          path: fullPath,
          kind: fs.existsSync(fullPath) ? 'modified' : 'deleted',
          content
        });

        for (const client of clients) {
          client.write(`event: file-changed\ndata: ${payload}\n\n`);
        }
      });
    } catch (e) {
      console.warn('Vite workspace fs.watch error:', e);
    }
  }

  return {
    name: 'yarac-workspace-fs',
    apply: 'serve',
    configureServer(server) {
      setupWatcher();

      // Ensure local Rust daemon is active on 127.0.0.1:9475
      const checkReq = http.get('http://127.0.0.1:9475/api/health', (healthRes) => {
        healthRes.resume();
      });
      checkReq.on('error', () => {
        const cliBinary = process.platform === 'win32'
          ? path.resolve(__dirname, '../target/debug/yarac-cli.exe')
          : path.resolve(__dirname, '../target/debug/yarac-cli');
        if (fs.existsSync(cliBinary)) {
          console.log('⚡ Launching local yarac-cli daemon on port 9475...');
          try {
            const daemonProc = child_process.spawn(cliBinary, ['serve', '--workspace', workspaceRoot, '--port', '9475'], {
              detached: true,
              stdio: 'ignore'
            });
            daemonProc.unref();
          } catch (spawnErr) {
            console.warn('Could not auto-spawn yarac-cli daemon:', spawnErr);
          }
        }
      });

      server.httpServer?.on('close', () => {
        if (fsWatcher) {
          fsWatcher.close();
          fsWatcher = null;
        }
      });

      server.middlewares.use(async (req, res, next) => {
        const url = req.url || '';

        // SSE Endpoint: /api/workspace/events
        if (url === '/api/workspace/events' && req.method === 'GET') {
          res.writeHead(200, {
            'Content-Type': 'text/event-stream',
            'Cache-Control': 'no-cache',
            'Connection': 'keep-alive',
            'Access-Control-Allow-Origin': '*'
          });
          clients.add(res);
          req.on('close', () => {
            clients.delete(res);
          });
          return;
        }

        // GET /api/workspace
        if (url === '/api/workspace' && req.method === 'GET') {
          res.setHeader('Content-Type', 'application/json');
          res.setHeader('Access-Control-Allow-Origin', '*');

          const manifestPath = path.join(workspaceRoot, 'workspace.json');
          let manifest = null;
          if (fs.existsSync(manifestPath)) {
            try {
              manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
            } catch {
              // ignore
            }
          }

          // Scan all .yarac files
          const files: Array<{ collectionName: string; fileName: string; name: string; path: string; dslContent: string }> = [];
          function scanDir(dir: string, colName: string) {
            if (!fs.existsSync(dir)) return;
            const entries = fs.readdirSync(dir, { withFileTypes: true });
            for (const entry of entries) {
              const full = path.join(dir, entry.name);
              if (entry.isDirectory()) {
                scanDir(full, entry.name);
              } else if (entry.name.endsWith('.yarac')) {
                try {
                  const content = fs.readFileSync(full, 'utf8');
                  const reqName = entry.name.replace(/\.yarac$/, '');
                  files.push({
                    collectionName: colName,
                    fileName: entry.name,
                    name: reqName,
                    path: full,
                    dslContent: content
                  });
                } catch {
                  // ignore
                }
              }
            }
          }

          scanDir(workspaceRoot, 'General');

          res.end(JSON.stringify({ manifest, requests: files }));
          return;
        }

        // POST /api/workspace/reveal
        if (url === '/api/workspace/reveal' && req.method === 'POST') {
          let bodyStr = '';
          req.on('data', chunk => {
            bodyStr += chunk;
          });
          req.on('end', () => {
            res.setHeader('Content-Type', 'application/json');
            res.setHeader('Access-Control-Allow-Origin', '*');

            try {
              const payload = JSON.parse(bodyStr || '{}');
              let targetPath = workspaceRoot;

              if (payload.path) {
                targetPath = path.isAbsolute(payload.path)
                  ? payload.path
                  : path.join(workspaceRoot, payload.path);
              } else if (payload.collectionName && payload.requestName) {
                const safeCol = (payload.collectionName || 'General').replace(/[/\\?%*:|"<>]/g, '_');
                const safeReq = (payload.requestName || 'Request').replace(/[/\\?%*:|"<>]/g, '_');
                targetPath = path.join(workspaceRoot, safeCol, `${safeReq}.yarac`);
                if (!fs.existsSync(targetPath)) {
                  targetPath = path.join(workspaceRoot, safeCol);
                }
              } else if (payload.collectionName) {
                const safeCol = (payload.collectionName || 'General').replace(/[/\\?%*:|"<>]/g, '_');
                targetPath = path.join(workspaceRoot, safeCol);
                if (!fs.existsSync(targetPath)) {
                  fs.mkdirSync(targetPath, { recursive: true });
                }
              }

              const resolved = path.resolve(targetPath);
              if (!resolved.startsWith(path.resolve(workspaceRoot))) {
                res.statusCode = 400;
                res.end(JSON.stringify({ error: 'Target path outside workspace root' }));
                return;
              }

              if (process.platform === 'win32') {
                if (fs.existsSync(resolved) && fs.statSync(resolved).isFile()) {
                  child_process.exec(`explorer.exe /select,"${resolved}"`);
                } else {
                  child_process.exec(`explorer.exe "${resolved}"`);
                }
              } else if (process.platform === 'darwin') {
                if (fs.existsSync(resolved) && fs.statSync(resolved).isFile()) {
                  child_process.spawn('open', ['-R', resolved], { detached: true });
                } else {
                  child_process.spawn('open', [resolved], { detached: true });
                }
              } else {
                const openTarget = fs.existsSync(resolved) && fs.statSync(resolved).isFile()
                  ? path.dirname(resolved)
                  : resolved;
                child_process.spawn('xdg-open', [openTarget], { detached: true });
              }

              res.end(JSON.stringify({ status: 'ok', path: resolved }));
            } catch (err: any) {
              res.statusCode = 500;
              res.end(JSON.stringify({ error: err.message }));
            }
          });
          return;
        }

        // POST /api/workspace/save
        if (url === '/api/workspace/save' && req.method === 'POST') {
          let bodyStr = '';
          req.on('data', chunk => {
            bodyStr += chunk;
          });
          req.on('end', () => {
            res.setHeader('Content-Type', 'application/json');
            res.setHeader('Access-Control-Allow-Origin', '*');

            try {
              const payload = JSON.parse(bodyStr);
              const manifestPath = path.join(workspaceRoot, 'workspace.json');

              // Suppress echo for manifest
              selfWriteTimes.set(manifestPath, Date.now() + 1500);
              fs.writeFileSync(manifestPath, JSON.stringify(payload.manifest, null, 2), 'utf8');

              const activeFilePaths = new Set<string>();
              const activeColDirs = new Set<string>();

              // Ensure collection folders exist for manifest collections
              if (Array.isArray(payload.manifest?.collections)) {
                for (const c of payload.manifest.collections) {
                  const safeCol = (c.name || 'General').replace(/[/\\?%*:|"<>]/g, '_');
                  const colDir = path.join(workspaceRoot, safeCol);
                  if (!fs.existsSync(colDir)) {
                    fs.mkdirSync(colDir, { recursive: true });
                  }
                  activeColDirs.add(colDir);
                }
              }

              // Write each request into collection folders
              if (Array.isArray(payload.requests)) {
                for (const r of payload.requests) {
                  const col = payload.manifest?.collections?.find((c: any) => c.id === r.collectionId);
                  const colName = (col?.name || 'General').replace(/[/\\?%*:|"<>]/g, '_');
                  const safeReqName = (r.name || 'Request').replace(/[/\\?%*:|"<>]/g, '_');

                  const colDir = path.join(workspaceRoot, colName);
                  if (!fs.existsSync(colDir)) {
                    fs.mkdirSync(colDir, { recursive: true });
                  }
                  activeColDirs.add(colDir);

                  const filePath = path.join(colDir, `${safeReqName}.yarac`);
                  activeFilePaths.add(filePath);
                  selfWriteTimes.set(filePath, Date.now() + 1500);

                  const content = r.dslContent || `version = "1.0"\nname = "${r.name}"\nmethod = "GET"\nurl = "http://localhost"\n`;
                  fs.writeFileSync(filePath, content, 'utf8');
                }
              }

              // Clean up orphan .yarac files that were deleted or renamed
              function cleanDir(dir: string) {
                if (!fs.existsSync(dir)) return;
                const entries = fs.readdirSync(dir, { withFileTypes: true });
                for (const entry of entries) {
                  const full = path.join(dir, entry.name);
                  if (entry.isDirectory()) {
                    cleanDir(full);
                    // If directory is not an active collection and is empty, remove it
                    if (!activeColDirs.has(full)) {
                      try {
                        const remaining = fs.readdirSync(full);
                        if (remaining.length === 0) {
                          fs.rmdirSync(full);
                        }
                      } catch {}
                    }
                  } else if (entry.name.endsWith('.yarac')) {
                    if (!activeFilePaths.has(full)) {
                      selfWriteTimes.set(full, Date.now() + 1500);
                      try {
                        fs.unlinkSync(full);
                      } catch {}
                    }
                  }
                }
              }
              cleanDir(workspaceRoot);

              res.end(JSON.stringify({ status: 'ok' }));
            } catch (err: any) {
              res.statusCode = 500;
              res.end(JSON.stringify({ error: err.message }));
            }
          });
          return;
        }

        next();
      });
    }
  };
}

export default defineConfig({
  plugins: [svelte(), yaracWorkspacePlugin()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      // Forward request/runner/interop execution to the local Rust daemon if running
      '/api/request': 'http://127.0.0.1:9475',
      '/api/runner': 'http://127.0.0.1:9475',
      '/api/interop': 'http://127.0.0.1:9475'
    }
  }
});
