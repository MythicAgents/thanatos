import glob
import os.path

commands = glob.glob(os.path.join(os.path.dirname(__file__), "*.py"))
__all__ = [
    os.path.basename(f)[:-3]
    for f in commands
    if os.path.isfile(f) and not f.endswith("__init__.py")
]

from . import *
