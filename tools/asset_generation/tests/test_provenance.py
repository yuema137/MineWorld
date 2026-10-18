"""A reference image outside the repository must not lose a paid image.

Regression: `build_provenance` called `Path.relative_to(repo_root)` on every reference *after* the paid
`images.edit` call, so a reference outside the repository raised `ValueError` and the image bytes were
never returned (four calibration images lost, 2026-10-15). The provider now reads every reference's
record before the request; outside the repository the record keeps the file name and the digest, never an
absolute path (a committed sidecar names no machine's directories).

Runs offline with a fake Images client, in CI's `python` layer:
    uv run --locked pytest tools/asset_generation/tests
"""

from __future__ import annotations

import base64
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from mineworld_gen import openai_images
from mineworld_gen.openai_images import OpenAIImageProvider
from mineworld_gen.provider import GenerationParams

PNG = b"\x89PNG\r\n\x1a\nfake-image-bytes"


class _FakeImages:
    def __init__(self) -> None:
        self.calls = 0

    def edit(self, image, **request):
        self.calls += 1
        return SimpleNamespace(
            data=[SimpleNamespace(b64_json=base64.b64encode(PNG).decode(), revised_prompt=None)],
            usage=None,
        )


class OutsideReferenceTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        tmp = Path(self.tmp.name)
        self.repo = tmp / "repo"
        (self.repo / "refs").mkdir(parents=True)
        self.inside = self.repo / "refs" / "anchor.png"
        self.inside.write_bytes(b"inside-reference")
        self.outside = tmp / "scratch" / "sheet.png"
        self.outside.parent.mkdir()
        self.outside.write_bytes(b"outside-reference")
        self._orig_root = openai_images._repo_root
        openai_images._repo_root = lambda: self.repo
        self.addCleanup(setattr, openai_images, "_repo_root", self._orig_root)
        self.provider = OpenAIImageProvider(model="test-model", api_key="not-a-real-key")
        self.fake = _FakeImages()
        self.provider._client = SimpleNamespace(images=self.fake)

    def test_outside_reference_keeps_the_image_and_records_its_name_and_digest_never_an_absolute_path(
        self,
    ) -> None:
        result = self.provider.generate(
            "a person", [self.inside, self.outside], GenerationParams(size="1024x1536")
        )
        self.assertEqual(self.fake.calls, 1)
        self.assertEqual(result.image_bytes, PNG)
        refs = result.provenance["generation"]["reference_images"]
        self.assertEqual(refs[0]["path"], "refs/anchor.png")
        self.assertEqual(refs[1]["path"], "(outside the repository)/sheet.png")
        self.assertNotIn(str(Path(self.tmp.name).resolve()), str(result.provenance))
        self.assertEqual(refs[1]["digest"], "sha256:" + hashlib.sha256(b"outside-reference").hexdigest())

    def test_missing_reference_fails_before_the_paid_call(self) -> None:
        with self.assertRaises(FileNotFoundError):
            self.provider.generate("a person", [self.outside.parent / "missing.png"])
        self.assertEqual(self.fake.calls, 0)


if __name__ == "__main__":
    unittest.main()
