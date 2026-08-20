import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

function createPNG(width, height) {
  // Signature
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

  // IHDR chunk
  const ihdrData = Buffer.alloc(13);
  ihdrData.writeUInt32BE(width, 0);
  ihdrData.writeUInt32BE(height, 4);
  ihdrData[8] = 8; // bit depth
  ihdrData[9] = 6; // color type: RGBA
  ihdrData[10] = 0; // compression method
  ihdrData[11] = 0; // filter method
  ihdrData[12] = 0; // interlace method
  const ihdrChunk = createChunk('IHDR', ihdrData);

  // Raw image data: height rows, each starting with filter byte 0, followed by width * 4 RGBA bytes
  const rowLength = 1 + width * 4;
  const rawData = Buffer.alloc(height * rowLength);
  
  // Fill with a sleek EmuBox Indigo/Cyan gradient
  for (let y = 0; y < height; y++) {
    const rowOffset = y * rowLength;
    rawData[rowOffset] = 0; // filter byte: None
    for (let x = 0; x < width; x++) {
      const pxOffset = rowOffset + 1 + x * 4;
      const r = Math.floor(99 + (156 - 99) * (x / width));   // #6366f1 (Indigo)
      const g = Math.floor(102 + (163 - 102) * (y / height));
      const b = Math.floor(241);
      const a = 255;
      rawData[pxOffset] = r;
      rawData[pxOffset + 1] = g;
      rawData[pxOffset + 2] = b;
      rawData[pxOffset + 3] = a;
    }
  }

  const compressedData = zlib.deflateSync(rawData);
  const idatChunk = createChunk('IDAT', compressedData);
  const iendChunk = createChunk('IEND', Buffer.alloc(0));

  return Buffer.concat([signature, ihdrChunk, idatChunk, iendChunk]);
}

function createChunk(type, data) {
  const length = data.length;
  const typeAndData = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = crc32(typeAndData);

  const chunk = Buffer.alloc(4 + 4 + length + 4);
  chunk.writeUInt32BE(length, 0);
  typeAndData.copy(chunk, 4);
  chunk.writeUInt32BE(crc, 4 + 4 + length);
  return chunk;
}

function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    c = (c >>> 8) ^ crcTable[(c ^ buf[i]) & 0xff];
  }
  return (c ^ 0xffffffff) >>> 0;
}

const crcTable = new Uint32Array(256);
for (let n = 0; n < 256; n++) {
  let c = n;
  for (let k = 0; k < 8; k++) {
    c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
  }
  crcTable[n] = c >>> 0;
}

const iconsDir = path.resolve('src-tauri', 'icons');
fs.mkdirSync(iconsDir, { recursive: true });

const icon512 = createPNG(512, 512);
const icon128 = createPNG(128, 128);
const icon32 = createPNG(32, 32);

fs.writeFileSync(path.join(iconsDir, 'icon.png'), icon512);
fs.writeFileSync(path.join(iconsDir, '128x128@2x.png'), icon512);
fs.writeFileSync(path.join(iconsDir, '128x128.png'), icon128);
fs.writeFileSync(path.join(iconsDir, '32x32.png'), icon32);
fs.writeFileSync(path.join(iconsDir, 'Square310x310Logo.png'), icon512);
fs.writeFileSync(path.join(iconsDir, 'Square150x150Logo.png'), icon128);
fs.writeFileSync(path.join(iconsDir, 'Square71x71Logo.png'), icon32);
fs.writeFileSync(path.join(iconsDir, 'Square44x44Logo.png'), icon32);

// Simple valid ICO header containing the 32x32 PNG
const icoHeader = Buffer.alloc(6);
icoHeader.writeUInt16LE(0, 0); // reserved
icoHeader.writeUInt16LE(1, 2); // type 1 = ICO
icoHeader.writeUInt16LE(1, 4); // 1 image

const icoDirEntry = Buffer.alloc(16);
icoDirEntry.writeUInt8(32, 0); // width
icoDirEntry.writeUInt8(32, 1); // height
icoDirEntry.writeUInt8(0, 2);  // color palette
icoDirEntry.writeUInt8(0, 3);  // reserved
icoDirEntry.writeUInt16LE(1, 4); // color planes
icoDirEntry.writeUInt16LE(32, 6); // bpp
icoDirEntry.writeUInt32LE(icon32.length, 8); // size
icoDirEntry.writeUInt32LE(22, 12); // offset (6 + 16 = 22)

const icoFile = Buffer.concat([icoHeader, icoDirEntry, icon32]);
fs.writeFileSync(path.join(iconsDir, 'icon.ico'), icoFile);

// Simple valid ICNS container
const icnsType = Buffer.from('icns');
const ic07Type = Buffer.from('ic07'); // 128x128 PNG icon
const ic07Chunk = Buffer.concat([ic07Type, Buffer.alloc(4), icon128]);
ic07Chunk.writeUInt32BE(ic07Chunk.length, 4);

const icnsFile = Buffer.concat([icnsType, Buffer.alloc(4), ic07Chunk]);
icnsFile.writeUInt32BE(icnsFile.length, 4);
fs.writeFileSync(path.join(iconsDir, 'icon.icns'), icnsFile);

console.log('[OK] Iconos nativos generados exitosamente en src-tauri/icons/');
