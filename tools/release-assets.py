"""Verify all platform artifacts, hash final assets, and preserve the highest latest version."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
package = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]
app, version = package["name"], package["version"]
dist = ROOT / "dist"
names = [f"{app}-{os_name}.zip" for os_name in ("windows", "mac", "linux")]
names += [f"{app}-installer-{kind}.{extension}" for kind, extension in (("exe", "exe"), ("dmg", "dmg"), ("deb", "deb"), ("appimage", "AppImage"))]
if app == "edot":
    # Transition assets allow existing manifest-based Windows clients to upgrade.
    names += ["edot-update.txt", f"Edot-Setup-{version}-windows-x64.exe"]
for name in names:
    path = dist / name
    if not path.is_file() or path.stat().st_size == 0:
        raise ValueError(f"Required release asset is missing: {name}")
for suffix in ("windows", "mac", "linux"):
    manifest = dist / f"SHA256SUMS-{suffix}.txt"
    for line in manifest.read_text(encoding="ascii").splitlines():
        digest, name = line.split("  ", 1)
        if name not in names or hashlib.sha256((dist / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"Artifact checksum mismatch: {name}")
    manifest.unlink()
(dist / "SHA256SUMS.txt").write_text("".join(f"{hashlib.sha256((dist / name).read_bytes()).hexdigest()}  {name}\n" for name in names), encoding="ascii", newline="\n")
result = subprocess.run(["gh", "api", f"repos/{os.environ['GITHUB_REPOSITORY']}/releases/latest"], capture_output=True, text=True)
if result.returncode:
    if "404" not in result.stderr:
        raise RuntimeError(result.stderr)
    latest = None
else:
    latest = json.loads(result.stdout)["tag_name"]
make_latest = True
if latest and re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", latest):
    make_latest = tuple(map(int, version.split("."))) > tuple(map(int, latest[1:].split(".")))
with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
    output.write(f"make_latest={str(make_latest).lower()}\n")
print(f"Verified {len(names)} release assets; make_latest={make_latest}")
