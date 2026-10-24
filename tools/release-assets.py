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
for name in names:
    path = dist / name
    if not path.is_file() or path.stat().st_size == 0:
        raise ValueError(f"Required release asset is missing: {name}")
metadata = [f"package-checksums-{suffix}.json" for suffix in ("windows", "mac", "linux")]
actual = {path.name for path in dist.iterdir()}
if actual != set(names + metadata):
    raise ValueError(f"Unexpected or missing release files: {actual.symmetric_difference(names + metadata)}")
verified = set()
for name in metadata:
    manifest = dist / name
    for asset, digest in json.loads(manifest.read_text(encoding="ascii")).items():
        if asset not in names or hashlib.sha256((dist / asset).read_bytes()).hexdigest() != digest:
            raise ValueError(f"Artifact checksum mismatch: {asset}")
        if asset in verified:
            raise ValueError(f"Duplicate checksum metadata: {asset}")
        verified.add(asset)
    manifest.unlink()
if verified != set(names):
    raise ValueError(f"Missing checksum metadata: {set(names) - verified}")
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
