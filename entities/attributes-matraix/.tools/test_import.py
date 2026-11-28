"""The import tool's own tests (S30-h1 §§5.2, 7; standard-library unittest, offline).

python3 -I -m unittest discover -s entities/attributes-matraix/.tools -p "test_*.py"
"""

from __future__ import annotations

import importlib.util
import json
import shutil
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("matraix_import", HERE / "import.py")
assert _spec is not None and _spec.loader is not None
tool = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(tool)


class Planted:
    """A scratch copy of the pack's inputs (`.upstream/`, the root files, the overlay) to plant faults in."""

    def __init__(self) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix="mineworld-import-test-")
        self.pack = Path(self.directory.name) / "pack"
        (self.pack / ".tools").mkdir(parents=True)
        shutil.copytree(tool.PACK / tool.UPSTREAM, self.pack / tool.UPSTREAM)
        for name in ("LICENSE", "CITATION.cff"):
            shutil.copyfile(tool.PACK / name, self.pack / name)
        self.overlay = tool.read_overlay()

    def write_overlay(self) -> None:
        (self.pack / ".tools" / "overlay.yaml").write_text(
            json.dumps(self.overlay), encoding="utf-8"
        )

    def rewrite_schema(self, change) -> None:
        """Changes the schema and pins its new digest, so only the planted fault is refused."""
        path = self.pack / tool.SCHEMA
        document = json.loads(path.read_text(encoding="utf-8"))
        change(document["dimensions"])
        path.write_text(json.dumps(document), encoding="utf-8")
        self.overlay["upstream"]["files"][tool.SCHEMA] = tool.sha256(path)

    def generate(self) -> dict[str, int]:
        self.write_overlay()
        out = Path(self.directory.name) / "out"
        return tool.generate(self.pack, out)

    def close(self) -> None:
        self.directory.cleanup()


class ImportTest(unittest.TestCase):
    def setUp(self) -> None:
        self.planted = Planted()

    def tearDown(self) -> None:
        self.planted.close()

    def refusal(self) -> str:
        with self.assertRaises(tool.ImportRefused) as raised:
            self.planted.generate()
        return str(raised.exception)

    def test_the_committed_pack_is_the_import_byte_for_byte(self) -> None:
        self.assertEqual(tool.check(tool.PACK), [])

    def test_the_fifty_excluded_dimensions_are_dropped_and_counted(self) -> None:
        dropped = self.planted.generate()
        self.assertEqual(dropped, {"citation-only": 28, "research-use-reference": 22})
        schema = json.loads((tool.PACK / tool.SCHEMA).read_text(encoding="utf-8"))
        excluded = {
            d["id"]
            for d in schema["dimensions"]
            if d.get("source_origin", {}).get("license") not in (None, "MIT")
        }
        self.assertEqual(len(excluded), 50)
        text = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (tool.PACK / "attributes").glob("*.yaml")
        )
        declared = {
            line.strip()[:-1]
            for line in text.splitlines()
            if line.startswith("  ") and not line.startswith("   ")
        }
        self.assertEqual(len(declared), 1240)
        self.assertFalse(excluded & declared)

    def test_one_excluded_dimension_let_through_fails_the_count(self) -> None:
        def let_through(dimensions: list[dict]) -> None:
            for dimension in dimensions:
                if "source_origin" in dimension:
                    del dimension["source_origin"]
                    return

        self.planted.rewrite_schema(let_through)
        self.assertTrue(self.refusal().startswith("ExcludedCount"))

    def test_one_changed_byte_upstream_is_a_provenance_mismatch(self) -> None:
        path = self.planted.pack / tool.SCHEMA
        data = bytearray(path.read_bytes())
        data[100] ^= 1
        path.write_bytes(bytes(data))
        refusal = self.refusal()
        self.assertTrue(refusal.startswith("ProvenanceMismatch"), refusal)
        self.assertIn(tool.SCHEMA, refusal)

    def test_a_value_slug_collision_is_refused(self) -> None:
        def collide(dimensions: list[dict]) -> None:
            dimensions[0]["values"][1] = dimensions[0]["values"][0].upper()

        self.planted.rewrite_schema(collide)
        self.assertTrue(self.refusal().startswith("ValueSlugCollision"))

    def test_an_untransliterated_character_is_refused(self) -> None:
        def accent(dimensions: list[dict]) -> None:
            dimensions[0]["values"][0] = "Naïve"

        self.planted.rewrite_schema(accent)
        self.assertTrue(self.refusal().startswith("UntransliteratedValue"))

    def test_slugs_keep_upstream_words_verbatim(self) -> None:
        table = self.planted.overlay["transliterate"]
        self.assertEqual(tool.slug("Early bird", table, "t"), "early-bird")
        self.assertEqual(tool.slug("Café regular", table, "t"), "cafe-regular")
        self.assertEqual(tool.slug("INTJ — Architect", table, "t"), "intj-architect")
        text = (tool.PACK / "attributes" / "behavior-time.yaml").read_text(
            encoding="utf-8"
        )
        self.assertIn('      early-bird: "Early bird"\n', text)
        self.assertIn('    phrase: "their sleep schedule is {value}"\n', text)

    def test_no_declaration_carries_a_default(self) -> None:
        for path in (tool.PACK / "attributes").glob("*.yaml"):
            self.assertNotIn("\n    default:", path.read_text(encoding="utf-8"), path.name)

    def test_generation_is_deterministic(self) -> None:
        self.planted.generate()
        first = tool.generated_files(Path(self.planted.directory.name) / "out")
        shutil.rmtree(Path(self.planted.directory.name) / "out")
        self.planted.generate()
        self.assertEqual(
            first, tool.generated_files(Path(self.planted.directory.name) / "out")
        )


if __name__ == "__main__":
    unittest.main()
