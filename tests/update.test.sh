#!/usr/bin/env bash
set -euo pipefail
project_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fixture_dir=$(mktemp -d)
trap 'rm -rf "$fixture_dir"' EXIT
mkdir -p "$fixture_dir/project/"{scripts,installer/lib,bin,.git} "$fixture_dir/commands" "$fixture_dir/logs"
cp "$project_dir/scripts/update-emubox.sh" "$fixture_dir/project/scripts/"
cp "$project_dir/installer/lib/architecture.sh" "$fixture_dir/project/installer/lib/"
source "$project_dir/installer/lib/architecture.sh"
case "$(get_emubox_architecture)" in
  x86_64) printf '\177ELF\002\001\001\000\000\000\000\000\000\000\000\000\002\000\076\000' > "$fixture_dir/native" ;;
  aarch64) printf '\177ELF\002\001\001\000\000\000\000\000\000\000\000\000\002\000\267\000' > "$fixture_dir/native" ;;
  *) exit 1 ;;
esac
cat > "$fixture_dir/commands/git" <<'GIT'
#!/usr/bin/env bash
case "$1" in
  status|fetch) exit 0 ;;
  rev-parse) printf '%s\n' "$2" ;;
  pull) printf 'wrong binary' > bin/emubox; [[ "$UPDATE_CASE" != pull_failure ]] ;;
  *) exit 1 ;;
esac
GIT
cat > "$fixture_dir/project/scripts/build.sh" <<'BUILD'
#!/usr/bin/env bash
cmp bin/emubox "$NATIVE_FIXTURE" || exit 4
case "$UPDATE_CASE" in
  build_failure) exit 1 ;;
  wrong_binary) printf 'wrong binary' > bin/emubox ;;
  success) cp "$NATIVE_FIXTURE" bin/emubox ;;
esac
BUILD
printf '#!/usr/bin/env bash\nexit 1\n' > "$fixture_dir/commands/systemctl"
chmod +x "$fixture_dir/commands/"*
export PATH="$fixture_dir/commands:$PATH"
export EMUBOX_LOG_DIR="$fixture_dir/logs"
export NATIVE_FIXTURE="$fixture_dir/native"
for UPDATE_CASE in pull_failure build_failure wrong_binary success; do
  export UPDATE_CASE
  cp "$NATIVE_FIXTURE" "$fixture_dir/project/bin/emubox"
  if bash "$fixture_dir/project/scripts/update-emubox.sh" > "$fixture_dir/output" 2>&1; then
    [[ "$UPDATE_CASE" == success ]]
  else
    [[ "$UPDATE_CASE" != success ]] || { cat "$fixture_dir/output"; exit 1; }
  fi
  cmp "$NATIVE_FIXTURE" "$fixture_dir/project/bin/emubox"
done
printf '%s\n' 'Update preserves native binary on pull/build/architecture failures: OK'