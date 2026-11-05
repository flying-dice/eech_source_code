"""Run an unchanged frozen regression-pack.py with an explicit native R: cwd.

PowerShell's provider location did not establish this for the LOS invocation;
its final report-display relpath failed across C:/R:. Scenario arguments and the
runner are unchanged. The real runner's SystemExit propagates to this process.
"""
import os
from pathlib import Path
import runpy
import sys

runner = Path(sys.argv[1]).absolute()
assert runner.name == 'regression-pack.py' and runner.is_file()
os.chdir('R:/d2-evidence')
assert os.path.splitdrive(os.getcwd())[0].lower() == 'r:'
print('NATIVE_PACK_CWD=' + os.getcwd(), flush=True)
sys.argv = [str(runner), *sys.argv[2:]]
runpy.run_path(str(runner), run_name='__main__')
