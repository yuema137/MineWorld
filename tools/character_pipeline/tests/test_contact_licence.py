"""I-S36-1: only `mwcast/contact.py` imports the mesh libraries, and nothing imports libigl's GPL part.

libigl's Python package ships `igl.copyleft`, which wraps GPL code (CGAL). DEP-142 adopts only the
MPL-2.0 core. The scan reads every Python file in the repository, so a copyleft
import cannot land anywhere unnoticed.
"""

from __future__ import annotations

import ast
import os
import sys
from pathlib import Path

import numpy as np
from conftest import PIPELINE_ROOT, REPO_ROOT
from contact_shapes import box

from mwcast import contact

LIBRARIES = ("igl", "trimesh", "fcl")
CONTACT_MODULE = PIPELINE_ROOT / "mwcast" / "contact.py"
# Build output, virtual environments and VCS data are not the repository's sources.
_SKIPPED_DIRECTORIES = {".git", "target", "node_modules", "__pycache__", ".venv", "venv", ".uv", ".tox"}


def _python_files() -> list[Path]:
    found: list[Path] = []
    for directory, subdirectories, files in os.walk(REPO_ROOT):
        subdirectories[:] = [
            d for d in subdirectories if d not in _SKIPPED_DIRECTORIES and not d.startswith(".venv")
        ]
        found.extend(Path(directory, name) for name in files if name.endswith(".py"))
    return found


def _imported_modules(tree: ast.AST) -> list[str]:
    """Every module a file imports: `import a.b`, `from a.b import c` (as a.b and a.b.c), and the string
    argument of `importlib.import_module(...)` / `__import__(...)`."""
    names: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            names.extend(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module is not None and node.level == 0:
            names.append(node.module)
            names.extend(f"{node.module}.{alias.name}" for alias in node.names)
        elif isinstance(node, ast.Call) and node.args:
            function = node.func
            called = (
                function.attr
                if isinstance(function, ast.Attribute)
                else function.id
                if isinstance(function, ast.Name)
                else ""
            )
            first = node.args[0]
            if called in ("import_module", "__import__") and isinstance(first, ast.Constant):
                if isinstance(first.value, str):
                    names.append(first.value)
    return names


def _is_copyleft(module: str) -> bool:
    return module == "igl.copyleft" or module.startswith("igl.copyleft.")


def _top(module: str) -> str:
    return module.split(".", 1)[0]


def _offences(files: list[Path]) -> tuple[list[str], list[str]]:
    copyleft: list[str] = []
    elsewhere: list[str] = []
    for path in files:
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        except (SyntaxError, UnicodeDecodeError):
            continue  # not a Python source this project runs (a template, a vendored fragment)
        for module in _imported_modules(tree):
            where = f"{path.relative_to(REPO_ROOT)}: {module}"
            if _is_copyleft(module):
                copyleft.append(where)
            if _top(module) in LIBRARIES and path.resolve() != CONTACT_MODULE.resolve():
                elsewhere.append(where)
    return copyleft, elsewhere


def test_no_file_imports_igl_copyleft_and_only_contact_py_imports_the_mesh_libraries() -> None:
    files = _python_files()
    assert CONTACT_MODULE in files
    copyleft, elsewhere = _offences(files)
    assert copyleft == [], f"GPL import of igl.copyleft (DEP-142): {copyleft}"
    assert elsewhere == [], f"a mesh library imported outside mwcast/contact.py (I-S36-1): {elsewhere}"


def test_the_scan_catches_each_form_of_a_copyleft_import(tmp_path: Path) -> None:
    sources = {
        "plain.py": "import igl.copyleft.cgal\n",
        "from_package.py": "from igl import copyleft\n",
        "from_module.py": "from igl.copyleft import cgal\n",
        "dynamic.py": "import importlib\nimportlib.import_module('igl.copyleft.tetgen')\n",
    }
    for name, text in sources.items():
        tree = ast.parse(text)
        assert any(_is_copyleft(m) for m in _imported_modules(tree)), name
    assert not any(
        _is_copyleft(m) for m in _imported_modules(ast.parse("import igl\nfrom igl import AABB\n"))
    )


def test_running_the_contact_module_loads_no_copyleft_module() -> None:
    vertices, faces = box((0.0, 0.0, 0.0), (1.0, 1.0, 1.0))
    cube = contact.TriMesh(vertices, faces)
    contact.inside(cube, [[0.5, 0.5, 0.5]])
    contact.outside(cube, [[2.0, 0.5, 0.5]])
    contact.lowest(vertices)
    contact.clearance(vertices, 0.0)
    moved = np.eye(4)
    moved[0, 3] = 0.5
    contact.overlaps([contact.RigidPart("a", cube, np.eye(4)), contact.RigidPart("b", cube, moved)])
    loaded = [name for name in sys.modules if _is_copyleft(name)]
    assert loaded == []
