"""Exercise the real helper without touching installed applications or user settings."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
APP = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']['name']
VERSION = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
WINDOWS = sys.platform == 'win32'

def wait_until(condition, process, message):
    deadline = time.monotonic() + 30
    while not condition():
        if process.poll() is not None or time.monotonic() > deadline:
            raise AssertionError(message)
        time.sleep(.03)

def compile_fixture(path, version):
    if WINDOWS:
        source = path.with_suffix('.cs')
        source.write_text('using System; using System.IO; using System.Threading; '
            f'[assembly:System.Reflection.AssemblyInformationalVersion("{version}")] '
            f'[assembly:System.Reflection.AssemblyFileVersion("{version}")] '
            'class Fixture { static void Main(string[] args) { '
            'if(args.Length>0) while(File.Exists(args[0])) Thread.Sleep(20); } }')
        compiler = Path(os.environ['WINDIR']) / 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
        subprocess.run([str(compiler), '/nologo', '/target:exe', '/out:' + str(path), str(source)], check=True, capture_output=True)
    else:
        path.write_text('#!/bin/sh\n# version=' + version + '\nexit 0\n')
        path.chmod(0o755)

def exercise(case):
    with tempfile.TemporaryDirectory(prefix=APP+'-helper-test-') as temporary:
        base = Path(temporary)
        # Both executable and update paths exercise spaces and apostrophes.
        install = base / "app O'Brien"
        install.mkdir()
        config = base / 'config'
        config.mkdir()
        task = base / (APP+'-update-' + ('1234567890abcdef1234567890abcdef' if APP == 'edot' else '1234567890'))
        task.mkdir()
        executable = install / (APP + ('.exe' if WINDOWS else ''))
        payload = base / ('new.exe' if WINDOWS else 'new')
        compile_fixture(executable, '1.0.0')
        compile_fixture(payload, VERSION)
        original = executable.read_bytes()
        marker = base / 'wait'
        marker.touch()
        parent = subprocess.Popen([str(executable), str(marker)], creationflags=subprocess.CREATE_NO_WINDOW) if WINDOWS else subprocess.Popen(['sleep','120'])
        archive = task / ('package.zip' if WINDOWS and APP == 'edot' else APP + ('-windows.zip' if WINDOWS else '-linux.zip'))
        with zipfile.ZipFile(archive,'w') as zipped:
            zipped.write(payload, executable.name)
            if APP == 'edolview':
                zipped.writestr('colormap/mono/gray.glsl', 'new colormap')
        if APP == 'edolview':
            (install/'colormap/mono').mkdir(parents=True)
            (install/'colormap/mono/gray.glsl').write_text('old colormap')
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        environment = os.environ.copy()
        prefix = APP.upper()
        for name,value in {'DIR':task,'EXE':executable,'PACKAGE':archive,'VERSION':VERSION,'HASH':digest,'PARENT':parent.pid,'SCOPE':'portable'}.items():
            environment[f'{prefix}_UPDATE_{name}'] = str(value)
        environment[prefix+'_CONFIG_DIR'] = str(config)
        environment[prefix+'_SPACE_DIR'] = str(base)
        context = dict(executable=str(executable),package=str(archive),version=VERSION,sha256=digest,parent=parent.pid,arguments=[],config=str(config))
        (task/'context.json').write_text(json.dumps(context))
        if WINDOWS:
            helper = task/'update.ps1'
            shutil.copyfile(ROOT/'installer/update.ps1', helper)
            if case == 'rollback':
                code = helper.read_text(encoding='utf-8')
                if APP == 'edot':
                    code = code.replace('[IO.File]::Copy($newExecutable, $executable, $true) }', "[IO.File]::Copy($newExecutable, $executable, $true); throw 'Injected copy failure' }")
                else:
                    code = code.replace('[IO.File]::Copy($files[$i].FullName, $rollback[$i].Destination, $true)', "[IO.File]::Copy($files[$i].FullName, $rollback[$i].Destination, $true); throw 'Injected copy failure'")
                helper.write_text(code, encoding='utf-8')
            shell = Path(os.environ['WINDIR'])/'System32/WindowsPowerShell/v1.0/powershell.exe'
            command = [str(shell),'-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-Command',
                '& ([ScriptBlock]::Create([IO.File]::ReadAllText($env:EDOL_HELPER_TEST_SCRIPT)))']
            environment['EDOL_HELPER_TEST_SCRIPT'] = str(helper)
        else:
            helper = task/'update.sh'
            shutil.copyfile(ROOT/'installer/update.sh', helper)
            (task/'restart.sh').write_text('#!/bin/sh\nexit 0\n')
            command = ['sh', str(helper)]
            if case == 'rollback':
                tools = base/'bin'
                tools.mkdir()
                # Fail only the staged EXE replacement, after its backup exists.
                # Restoring the backup uses .old and remains permitted.
                mv = tools/'mv'
                mv.write_text('#!/bin/sh\ncase "$*" in *.update.*) exit 77 ;; esac\nexec /bin/mv "$@"\n')
                mv.chmod(0o755)
                environment['PATH'] = str(tools) + os.pathsep + environment['PATH']
        if case == 'bad-download':
            archive.write_bytes(archive.read_bytes()+b'corruption')
        child = subprocess.Popen(command, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            **({'creationflags':subprocess.CREATE_NO_WINDOW} if WINDOWS else {}))
        try:
            if case == 'bad-download':
                child.communicate(timeout=30)
                assert not (task/'ready').exists()
            else:
                wait_until(lambda:(task/'ready').is_file(),child,'helper did not become ready')
                if case == 'tampered-after-ready':
                    archive.write_bytes(archive.read_bytes()+b'changed after verification')
                if case != 'cancel':
                    (task/'go').write_text('install')
                parent.terminate()
                parent.wait(timeout=10)
                child.communicate(timeout=30)
            if parent.poll() is None:
                parent.terminate()
                parent.wait(timeout=10)
            if WINDOWS:
                deadline = time.monotonic() + 10
                time.sleep(.2)
                while True:
                    try:
                        with executable.open('r+b'): pass
                        break
                    except PermissionError:
                        if time.monotonic() > deadline: raise
                        time.sleep(.05)
            if case == 'success':
                assert executable.read_bytes() == payload.read_bytes()
                assert not (config/'update-result.txt').exists() or not (config/'update-result.txt').read_text(encoding="utf-8").strip()
                if APP == 'edolview': assert (install/'colormap/mono/gray.glsl').read_text(encoding="utf-8") == 'new colormap'
            else:
                assert executable.read_bytes() == original, case + ': original executable was not preserved'
                if APP == 'edolview': assert (install/'colormap/mono/gray.glsl').read_text(encoding="utf-8") == 'old colormap'
                if case != 'cancel': assert (config/'update-result.txt').read_text(encoding="utf-8").strip()
            print('PASS',APP,sys.platform,case,flush=True)
        finally:
            for process in [parent,child]:
                if process.poll() is None:
                    process.kill()
                    process.wait()

for case in ['bad-download','cancel','tampered-after-ready','success'] + ['rollback']:
    exercise(case)
