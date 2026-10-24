"""Run actual signed DMG replacements and rollback in an isolated writable folder."""
import hashlib
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile
import time
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = tomllib.loads((ROOT/'Cargo.toml').read_text())['package']
APP, VERSION = PACKAGE['name'], PACKAGE['version']
NAME = {'edolview':'Edolview','edolwave':'EdolWave','edot':'Edot'}[APP]

def run(*args):
    return subprocess.run(list(map(str,args)),check=True,capture_output=True)

with tempfile.TemporaryDirectory(prefix=APP+'-mac-test-') as temporary:
    base=Path(temporary)
    source=base/'source'/f'{NAME}.app'
    (source/'Contents/MacOS').mkdir(parents=True)
    shutil.copy2('/usr/bin/true',source/'Contents/MacOS'/APP)
    (source/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'dev.edoli.helperfixture','CFBundleExecutable':APP,'CFBundlePackageType':'APPL','CFBundleVersion':VERSION,'CFBundleShortVersionString':VERSION}))
    run('codesign','--force','--deep','--sign','-',source)
    dmg=base/'fixture.dmg'
    run('hdiutil','create','-srcfolder',source.parent,'-format','UDZO',dmg)
    for case in ['success','rollback']:
        target=base/case/f'{NAME}.app'
        target.parent.mkdir()
        shutil.copytree(source,target)
        (target/'old-marker').write_text('old bundle')
        task=base/(APP+'-update-'+case)
        task.mkdir()
        config=task/'config'
        config.mkdir()
        package=task/(APP+'-installer-dmg.dmg')
        shutil.copyfile(dmg,package)
        (task/'restart.sh').write_text('#!/bin/sh\nexit 0\n')
        (task/'update.sh').write_text((ROOT/'installer/update.sh').read_text())
        parent=subprocess.Popen(['sleep','120'])
        env=os.environ.copy()
        for key,value in {'DIR':task,'EXE':target/'Contents/MacOS'/APP,'PACKAGE':package,'VERSION':VERSION,'HASH':hashlib.sha256(package.read_bytes()).hexdigest(),'PARENT':parent.pid}.items():
            env[APP.upper()+'_UPDATE_'+key]=str(value)
        env[APP.upper()+'_CONFIG_DIR']=str(config)
        if case=='rollback':
            # Inject a failure into this test's private helper copy after the old
            # bundle was moved aside, then exercise its real cleanup transaction.
            helper=task/'update.sh'
            helper.write_text(helper.read_text().replace('mv "$work/new.app" "$target_app"', 'exit 79'))
        child=subprocess.Popen(['sh',str(task/'update.sh')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            deadline=time.monotonic()+30
            while not (task/'ready').exists():
                assert child.poll() is None and time.monotonic()<deadline,'helper not ready'
                time.sleep(.05)
            (task/'go').touch()
            parent.terminate()
            parent.wait()
            child.communicate(timeout=90)
            assert target.is_dir()
            if case=='success':
                assert not (target/'old-marker').exists()
                run('codesign','--verify','--deep','--strict',target)
            else:
                assert (target/'old-marker').read_text()=='old bundle'
                assert (config/'update-result.txt').read_text().strip()
            print('PASS',APP,'macOS DMG',case,flush=True)
        finally:
            for process in [parent,child]:
                if process.poll() is None:
                    process.kill()
                    process.wait()
