from pathlib import Path
import shutil, subprocess
out=Path('E:/eech-runs/m5-d1/aircraft-control-20261023/coverage')
root=out/'lebanon-root'
out.mkdir(exist_ok=True)
shutil.copytree('E:/eech-runs/m2/lebanon-root',root)
w=Path('E:/eech_source_code/.claude/worktrees/m5-aircraft-control/eech-campaign')
args=[str(w/'target/aircraft-control/eech-world.exe'),str(w/'crates/eech-world/lua/fixed-wing-movement.lua'),'root='+root.as_posix(),'mode=census','hours=3']
with (out/'census.out').open('wb') as stdout,(out/'census.err').open('wb') as stderr:
    p=subprocess.Popen(args,cwd=w/'target/aircraft-control',stdout=stdout,stderr=stderr)
    (out/'census.pid').write_text(str(p.pid))
    rc=p.wait()
(out/'census.exit').write_text(str(rc))
print('census exit',rc)
