"""Validate final portable payloads, target architecture, and package checksums."""
import hashlib
import json
import tempfile
import os
from pathlib import Path
import plistlib
import re
import struct
import subprocess
import sys
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "target/dist"
package = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]
app, version = package["name"], package["version"]
suffix = {"win32": "windows", "darwin": "mac", "linux": "linux"}[sys.platform]
with zipfile.ZipFile(DIST / f"{app}-{suffix}.zip") as archive:
    names = archive.namelist()
    if suffix == "mac":
        display = {"edolview": "Edolview", "edolwave": "EdolWave", "edot": "Edot"}[app]
        prefix = f"{display}.app/Contents/"
        assert all(name.startswith(f"{display}.app/") for name in names), names
        info = plistlib.loads(archive.read(prefix + "Info.plist"))
        assert info["CFBundleVersion"] == version
        assert info["CFBundleExecutable"] == app
        binary = archive.read(prefix + f"MacOS/{app}")
        assert binary[:4] == b"\xcf\xfa\xed\xfe" and struct.unpack_from("<I", binary, 4)[0] == 0x100000C, "Expected arm64 Mach-O"
        with tempfile.TemporaryDirectory() as extracted:
            subprocess.run(["ditto", "-x", "-k", str(DIST / f"{app}-mac.zip"), extracted], check=True)
            subprocess.run(["codesign", "--verify", "--deep", "--strict", str(Path(extracted) / f"{display}.app")], check=True)
    else:
        executable = app + (".exe" if suffix == "windows" else "")
        assert executable in names
        assert all(name == executable or (app == "edolview" and name.startswith("colormap/")) for name in names), names
        binary = archive.read(executable)
        if suffix == "linux":
            assert archive.getinfo(executable).external_attr >> 16 & 0o111, "Executable permission missing"
            assert binary[:4] == b"\x7fELF" and struct.unpack_from("<H", binary, 18)[0] == 62, "Expected x64 ELF"
        else:
            pe = struct.unpack_from("<I", binary, 0x3C)[0]
            assert binary[pe:pe + 4] == b"PE\0\0" and struct.unpack_from("<H", binary, pe + 4)[0] == 0x8664
            count = struct.unpack_from("<H", binary, pe + 6)[0]
            optional_size = struct.unpack_from("<H", binary, pe + 20)[0]
            optional = pe + 24
            assert struct.unpack_from("<H", binary, optional)[0] == 0x20B
            sections = optional + optional_size
            def offset(rva):
                for index in range(count):
                    base = sections + 40 * index
                    size, virtual, raw_size, raw = struct.unpack_from("<IIII", binary, base + 8)
                    if virtual <= rva < virtual + max(size, raw_size):
                        return raw + rva - virtual
                raise ValueError(f"Unmapped PE address: {rva:x}")
            imports = offset(struct.unpack_from("<I", binary, optional + 120)[0])
            dlls = []
            while any(binary[imports:imports + 20]):
                name_offset = offset(struct.unpack_from("<I", binary, imports + 12)[0])
                dlls.append(binary[name_offset:binary.index(0, name_offset)].decode("ascii"))
                imports += 20
            assert not any(re.search(r"^(vcruntime|msvcp|concrt)|^api-ms-win-crt-", dll, re.I) for dll in dlls), dlls
            print("Verified x64 PE with static CRT:", ", ".join(dlls))
if suffix == "linux":
    path = DIST / f"{app}-installer-deb.deb"
    assert subprocess.check_output(["dpkg-deb", "-f", str(path), "Version"], text=True).strip() == version
    assert subprocess.check_output(["dpkg-deb", "-f", str(path), "Architecture"], text=True).strip() == "amd64"
    listing = subprocess.check_output(["dpkg-deb", "-c", str(path)], text=True)
    assert f"./usr/bin/{app}" in listing and f"./opt/{app}" not in listing, listing
    assert not re.search(r"/(README(?:\.md)?|LICENSE)(?:\.gz)?$", listing, re.M), listing
    assert (DIST / f"{app}-installer-appimage.AppImage").read_bytes()[:4] == b"\x7fELF"
manifest = DIST / f"package-checksums-{suffix}.json"
for name, digest in json.loads(manifest.read_text(encoding="ascii")).items():
    assert hashlib.sha256((DIST / name).read_bytes()).hexdigest() == digest, name
print(f"Verified {app} {version} {suffix} packages and checksums")
