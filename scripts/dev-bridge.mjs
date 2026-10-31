#!/usr/bin/env node
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { exec } from 'node:child_process';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const PORT = process.env.PORT || 9191;

// Path to sqlite db
const dbCandidates = [
  process.env.EMUBOX_DB_PATH,
  '/var/lib/emubox/emubox.db',
  path.join(root, 'data', 'emubox.db'),
  path.join(root, 'emubox.db'),
].filter(Boolean);

let db = null;
for (const p of dbCandidates) {
  if (fs.existsSync(p)) {
    try {
      const { DatabaseSync } = await import('node:sqlite');
      db = new DatabaseSync(p);
      console.log(`[DevBridge] Conectado a base de datos SQLite: ${p}`);
      break;
    } catch (e) {
      console.warn(`[DevBridge] No se pudo abrir ${p}: ${e.message}`);
    }
  }
}

const server = http.createServer((req, res) => {
  // CORS
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  const url = new URL(req.url, `http://${req.headers.host || 'localhost'}`);

  if (url.pathname === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'running', vm: true, hasDb: Boolean(db), timestamp: Date.now() }));
    return;
  }

  if (url.pathname === '/api/games' && req.method === 'GET') {
    if (!db) {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify([]));
      return;
    }
    try {
      const rows = db.prepare('SELECT id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, play_time_minutes, favorite, cover_image, backdrop_image, description, rom_path, file_size_bytes FROM games').all();
      const games = rows.map(r => ({
        id: r.id,
        title: r.title,
        platform: r.platform_id,
        platformName: r.platform_name,
        releaseYear: r.release_year,
        genre: r.genre || 'Desconocido',
        developer: r.developer || '',
        publisher: r.publisher || '',
        rating: r.rating || 0,
        playTimeMinutes: r.play_time_minutes || 0,
        favorite: Boolean(r.favorite),
        coverImage: r.cover_image || '',
        backdropImage: r.backdrop_image || '',
        description: r.description || '',
        romPath: r.rom_path || '',
        fileSizeBytes: r.file_size_bytes || 0,
        installed: Boolean(r.rom_path),
      }));
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(games));
    } catch (err) {
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: err.message }));
    }
    return;
  }

  
  if (url.pathname === '/api/platforms' && req.method === 'GET') {
    if (!db) {
      const fallbackPlatforms = path.join(root, 'data', 'platforms.json');
      if (fs.existsSync(fallbackPlatforms)) {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(fs.readFileSync(fallbackPlatforms, 'utf8'));
        return;
      }
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify([]));
      return;
    }
    try {
      const rows = db.prepare('SELECT id, name, short_name, manufacturer, generation, release_year, color, icon, default_emulator_id FROM systems').all();
      const platforms = rows.map(r => ({
        id: r.id,
        name: r.name,
        shortName: r.short_name,
        manufacturer: r.manufacturer,
        generation: r.generation,
        releaseYear: r.release_year,
        color: r.color,
        icon: r.icon,
        defaultEmulatorId: r.default_emulator_id,
      }));
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(platforms));
    } catch (err) {
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: err.message }));
    }
    return;
  }

  if (url.pathname === '/api/emulators' && req.method === 'GET') {
    if (!db) {
      const fallbackEmus = path.join(root, 'data', 'emulators.json');
      if (fs.existsSync(fallbackEmus)) {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(fs.readFileSync(fallbackEmus, 'utf8'));
        return;
      }
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify([]));
      return;
    }
    try {
      const rows = db.prepare('SELECT id, official_name, version, supported_platforms_json, core_type, status, executable_path, default_arguments_json FROM emulators').all();
      const emus = rows.map(r => ({
        id: r.id,
        name: r.official_name,
        version: r.version,
        supportedPlatforms: JSON.parse(r.supported_platforms_json || '[]'),
        coreType: r.core_type,
        status: r.status,
        executable: r.executable_path,
        arguments: JSON.parse(r.default_arguments_json || '[]'),
      }));
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(emus));
    } catch (err) {
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: err.message }));
    }
    return;
  }

  if (url.pathname === '/api/launch' && req.method === 'POST') {
    let body = '';
    req.on('data', chunk => { body += chunk; });
    req.on('end', () => {
      try {
        const { gameId, emulatorId } = JSON.parse(body || '{}');
        if (!gameId) {
          res.writeHead(400, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ error: 'Falta gameId' }));
          return;
        }
        console.log(`[DevBridge] Solicitud de lanzamiento de juego en la MV: ${gameId} (emulador: ${emulatorId || 'auto'})`);
        if (process.platform === 'linux') {
          const cmd = `/usr/local/bin/emubox-session launch "${gameId}"`;
          exec(cmd, (error) => {
            if (error) console.error(`[DevBridge] Error al lanzar en consola: ${error.message}`);
          });
        }
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
          success: true,
          pid: process.pid,
          message: `Juego ${gameId} enviado para ejecución en la pantalla de la Máquina Virtual`,
        }));
      } catch (err) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: err.message }));
      }
    });
    return;
  }

  res.writeHead(404, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ error: 'Ruta no encontrada' }));
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`======================================================================`);
  console.log(`🎮 [EmuBox Dev Bridge] Servidor de desarrollo activo en http://0.0.0.0:${PORT}`);
  console.log(`   Permite conectar el navegador en el host con la Máquina Virtual.`);
  console.log(`======================================================================`);
});

