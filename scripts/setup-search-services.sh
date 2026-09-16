#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ "$EUID" -ne 0 ]]; then
  printf 'Ejecuta este instalador como administrador. No inicia descargas ni cambia TTY1.\n' >&2
  exit 1
fi
getent group emubox >/dev/null
id emubox >/dev/null
pacman -S --needed qbittorrent-nox coreutils util-linux libarchive nodejs
if [[ ! -e /opt/jackett ]]; then
  STAGING=$(mktemp -d /var/tmp/emubox-jackett-install-XXXXXXXX)
  trap 'rm -rf -- "$STAGING"' EXIT
  if [[ -n "${JACKETT_ARCHIVE:-}" ]]; then
    if [[ ! "${JACKETT_SHA256:-}" =~ ^[a-fA-F0-9]{64}$ || ! -f "$JACKETT_ARCHIVE" ]]; then
      printf 'El archivo local requiere JACKETT_SHA256 esperado de una fuente verificada.\n' >&2
      exit 1
    fi
    ACTUAL_SHA256=$(sha256sum -- "$JACKETT_ARCHIVE")
    if [[ "${ACTUAL_SHA256%% *}" != "${JACKETT_SHA256,,}" ]]; then printf 'SHA-256 local no coincide.\n' >&2; exit 1; fi
    cp -- "$JACKETT_ARCHIVE" "$STAGING/jackett.tar.gz"
  else
  node --input-type=module - "$STAGING" <<'NODE'
import fs from 'node:fs';
import { createHash } from 'node:crypto';
const staging=process.argv[2];
const assetName={x64:'Jackett.Binaries.LinuxAMDx64.tar.gz',arm64:'Jackett.Binaries.LinuxARM64.tar.gz'}[process.arch];
if(!assetName) throw new Error('Arquitectura Jackett no admitida');
const response=await fetch('https://api.github.com/repos/Jackett/Jackett/releases/latest',{
  headers:{'User-Agent':'EmuBox-local-setup','Accept':'application/vnd.github+json'},signal:AbortSignal.timeout(30000)});
if(!response.ok) throw new Error(`Publicacion Jackett HTTP ${response.status}`);
const release=await response.json();
const asset=release.assets?.find(item=>item.name===assetName);
if(!asset || !/^sha256:[a-f0-9]{64}$/.test(asset.digest??'') || asset.size>256*1024*1024) throw new Error('Artefacto sin SHA-256 verificado o demasiado grande');
const url=new URL(asset.browser_download_url);
if(url.protocol!=='https:'||url.hostname!=='github.com'||!url.pathname.startsWith('/Jackett/Jackett/releases/download/')) throw new Error('Origen de artefacto no autorizado');
const archive=await fetch(url,{signal:AbortSignal.timeout(180000)});
if(!archive.ok||!archive.body) throw new Error(`Artefacto Jackett HTTP ${archive.status}`);
const file=fs.openSync(`${staging}/jackett.tar.gz`,'wx',0o600);
const hash=createHash('sha256');let size=0;
try {
  for await(const chunk of archive.body) {
    size+=chunk.length;if(size>256*1024*1024)throw new Error('Artefacto excede limite');
    hash.update(chunk);let written=0;while(written<chunk.length)written+=fs.writeSync(file,chunk,written);
  }
} finally {fs.closeSync(file);}
if(size!==asset.size||`sha256:${hash.digest('hex')}`!==asset.digest)throw new Error('Huella o tamano Jackett no coincide');
console.log(`Jackett ${release.tag_name}: artefacto oficial verificado para ${process.arch}`);
NODE
  fi
  bsdtar -tf "$STAGING/jackett.tar.gz" | node --input-type=module -e '
let text="";for await(const data of process.stdin)text+=data;
for(const name of text.trim().split("\n"))if(!name.startsWith("Jackett/")||name.split("/").includes(".."))throw new Error("Ruta de archivo Jackett invalida");'
  install -d -m 0700 "$STAGING/extracted"
  bsdtar --no-same-owner --no-same-permissions -xf "$STAGING/jackett.tar.gz" -C "$STAGING/extracted"
  if [[ ! -f "$STAGING/extracted/Jackett/jackett" || -n "$(find "$STAGING/extracted" -type l -print -quit)" ]]; then
    printf 'Distribucion Jackett contiene enlaces o estructura inesperada.\n' >&2
    exit 1
  fi
  chown -R root:root "$STAGING/extracted/Jackett"
  chmod -R go-w "$STAGING/extracted/Jackett"
  chmod 0755 "$STAGING/extracted/Jackett/jackett"
  mv "$STAGING/extracted/Jackett" /opt/jackett
fi
if [[ ! -x /opt/jackett/jackett ]]; then
  printf 'Instalacion Jackett existente incompleta; no se sobrescribe.\n' >&2
  exit 1
fi
if [[ -L /opt/jackett || -L /opt/jackett/jackett ]]; then
  printf 'La ruta de Jackett no puede ser un enlace simbolico.\n' >&2
  exit 1
fi
if [[ -n "$(find /opt/jackett -type f \( ! -user root -o -perm /022 \) -print -quit)" ]]; then
  printf 'La distribucion Jackett debe pertenecer a root sin escritura de grupo/otros.\n' >&2
  exit 1
fi
if ! id emubox-jackett >/dev/null 2>&1; then
  useradd --system --gid emubox --home-dir /var/lib/emubox/jackett --shell /usr/bin/nologin emubox-jackett
fi
if systemctl is-active --quiet emubox-jackett.service; then
  printf 'Deten emubox-jackett.service antes de reconfigurarlo.\n' >&2
  exit 1
fi
umask 0027
for DIRECTORY in /var/lib/emubox/jackett /var/lib/emubox/jackett/Jackett; do
  if [[ -L "$DIRECTORY" ]]; then printf 'Ruta de estado enlazada: %s\n' "$DIRECTORY" >&2; exit 1; fi
done
install -d -m 0750 -o emubox-jackett -g emubox /var/lib/emubox/jackett /var/lib/emubox/jackett/Jackett
node --input-type=module <<'NODE'
import fs from 'node:fs';
import { randomBytes, createHash } from 'node:crypto';
const root='/var/lib/emubox/jackett';
const path=`${root}/Jackett/ServerConfig.json`;
for(const candidate of [root,`${root}/Jackett`,path,`${root}/admin-password`]) {
  if(fs.existsSync(candidate)&&fs.lstatSync(candidate).isSymbolicLink()) throw new Error('Ruta de configuracion enlazada');
}
const config=fs.existsSync(path)?JSON.parse(fs.readFileSync(path,'utf8')):{};
config.APIKey ||= randomBytes(32).toString('hex');
if(!config.AdminPassword) {
  const password=randomBytes(24).toString('base64url');
  fs.writeFileSync(`${root}/admin-password`,password+'\n',{mode:0o640,flag:'wx'});
  config.AdminPassword=createHash('sha512').update(Buffer.from(password+config.APIKey,'utf16le')).digest('hex');
}
Object.assign(config,{Port:9117,LocalBindAddress:'127.0.0.1',AllowExternal:false,AllowCORS:false,UpdateDisabled:true});
fs.writeFileSync(path,JSON.stringify(config,null,2)+'\n',{mode:0o640});
NODE
chown emubox-jackett:emubox /var/lib/emubox/jackett/Jackett/ServerConfig.json
chmod 0640 /var/lib/emubox/jackett/Jackett/ServerConfig.json
if [[ -f /var/lib/emubox/jackett/admin-password ]]; then
  chown emubox-jackett:emubox /var/lib/emubox/jackett/admin-password
  chmod 0640 /var/lib/emubox/jackett/admin-password
fi
install -m 0644 "$ROOT_DIR/installer/config/emubox-jackett.service" /etc/systemd/system/emubox-jackett.service
systemctl daemon-reload
systemctl enable --now emubox-jackett.service
printf 'Jackett local: http://127.0.0.1:9117. Configura tus indexadores autorizados.\n'
printf 'Contraseña administrativa guardada en /var/lib/emubox/jackett/admin-password; no compartir en logs ni chat.\n'