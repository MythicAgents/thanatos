import glob
import os.path
import itertools
from pathlib import Path
from importlib import import_module, invalidate_caches
import sys

# Get file paths of all modules.

currentPath = Path(__file__)
searchPaths = [
    currentPath.parent / "agent_functions" / "*.py",
]

modules = map(
    lambda x: Path(x),
    itertools.chain.from_iterable([glob.glob(f"{pattern}") for pattern in searchPaths]),
)

modules = filter(lambda x: x.name != "__init__.py", modules)
modules = [p.parts[p.parts.index("agent_functions") :] for p in modules]
invalidate_caches()
for module in modules:
    module = [component.removesuffix(".py") for component in module]
    import_name = "{}.{}".format(__name__, ".".join(module))
    print(import_name)
    m = import_module(import_name)
    for el in dir(module):
        if "__" not in el:
            globals()[el] = getattr(module, el)


sys.path.append(os.path.abspath(currentPath.name))
