"""The review pages of the pilot (design §4.9; I-p-4, I-p-13; AC-p-1, AC-p-10): `VIS-3D-WARDROBE-P` and the
pre-spend page. Stdlib only (a frame is converted to JPEG when Pillow is importable, else copied as PNG).

Rules the generator enforces, not the author:

- **Verdict honesty (I-p-13).** The header states the decision rule. Unless the page data's status is `passed`
  and every subject has both outfits' sheets, the page leads with the state it is in: `failed` leads with
  "route B 未达到 v2/Vera 观感，不采用" and the levers; `pending` and `not-run` say so. Nothing is labelled
  accepted: acceptance is the operator's verdict.
- **No stock asset as the look (I-p-4).** An outfit whose V-1 is not PASS has its frames withheld: the page
  shows the refusal and the reason, never the frames.
- **Red frames stay red.** A flagged frame is shown with its red border and its checks, first-class.

The page is written in Chinese with English terms (the operator's language); paths are relative inside the
output folder, which lives outside the repository (`mineworld-demos/wardrobe-pilot/`).
"""

from __future__ import annotations

import html
import json
import shutil
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import cast

NOT_ADOPTED = "route B 未达到 v2/Vera 观感，不采用"
DECISION_RULE = (
    "Route B 只有在每个 subject 的每套 outfit 都通过 W-1…W-12 与 G-1…G-14、达到 S26 scorecard 标准，"
    "并且由 operator 在 fused Vera 旁判定达到 v2/Vera 观感（两套 outfit 都要）时才被采用；"
    "否则不采用，并给出诊断与 R-S32-1 的 levers（step-32 §5.2；ARC-23 rule 2）。"
)
STATUS_LINES = {
    "passed": (
        "所有 gate 已通过：请 operator 判定 — route B 达到 v2/Vera 的观感了吗？"
        "（accept / reject，每个 subject 与总体）"
    ),
    "pending": "Pilot 尚未完成：下面是已有的部分，结论待定（pending VIS-3D-WARDROBE-P）。",
    "not-run": "Pilot 的付费阶段尚未运行：本页只展示零成本的验证（fixtures），不代表 route B 的观感。",
}


class PageError(Exception):
    """The page data is malformed."""


def _e(value: object) -> str:
    return html.escape(str(value))


def _copy_image(source: Path, out: Path, name: str) -> str:
    """Copy (as JPEG when Pillow is present) into `out/img/`, returning the relative path."""
    (out / "img").mkdir(parents=True, exist_ok=True)
    try:
        from PIL import Image  # pyright: ignore[reportMissingImports, reportUnknownVariableType]

        target = out / "img" / f"{name}.jpg"
        Image.open(source).convert("RGB").save(target, quality=88)  # pyright: ignore
    except Exception:  # noqa: BLE001 - no Pillow, or a file it cannot read: the original bytes are copied
        target = out / "img" / f"{name}{source.suffix}"
        shutil.copyfile(source, target)
    return f"img/{target.name}"


def _list(value: object) -> list[Mapping[str, object]]:
    if not isinstance(value, list):
        return []
    return [cast(Mapping[str, object], v) for v in cast(list[object], value) if isinstance(v, dict)]


def _frames_html(outfit: Mapping[str, object], out: Path, prefix: str) -> str:
    v1 = str(outfit.get("v1", "NOT RUN"))
    if v1 != "PASS":
        return (
            f'<p class="refused">V-1 {_e(v1)}: 此 outfit 的画面不作为观感展示（I-p-4）。'
            f"{_e(outfit.get('v1_reason', ''))}</p>"
        )
    cards: list[str] = []
    for frame in _list(outfit.get("frames")):
        source = Path(str(frame["path"]))
        rel = _copy_image(source, out, f"{prefix}-{frame['id']}")
        ok = bool(frame.get("pass", False))
        checks = _e(json.dumps(frame.get("checks", {}), ensure_ascii=False))
        cards.append(
            f'<figure class="{"pass" if ok else "flagged"}"><a href="{rel}">'
            f'<img src="{rel}" alt="{_e(frame["id"])}"></a>'
            f"<figcaption><b>{_e(frame['id'])}</b> {'PASS' if ok else 'FLAGGED'}<br><code>{checks}</code>"
            "</figcaption></figure>"
        )
    return '<div class="grid">' + "".join(cards) + "</div>"


def _gates_html(outfit: Mapping[str, object]) -> str:
    rows = "".join(
        f'<tr class="{_e(str(g.get("verdict", "")).lower())}"><td>{_e(g.get("gate"))}</td>'
        f"<td>{_e(g.get('verdict'))}</td><td>{_e(g.get('where', ''))}</td></tr>"
        for g in _list(outfit.get("gates"))
    )
    return (
        f"<table class='gates'><tr><th>gate</th><th>verdict</th><th>where it looked</th></tr>{rows}</table>"
    )


def render(data: Mapping[str, object], out: Path) -> Path:
    """Write `out/index.html` from the page data (see the module docstring for the rules)."""
    status = str(data.get("status", "pending"))
    subjects = _list(data.get("subjects"))
    complete = bool(subjects) and all(
        len([o for o in _list(s.get("outfits")) if _list(o.get("frames"))]) >= 2 for s in subjects
    )
    if status == "passed" and not complete:
        status = "pending"  # a "passed" page without every subject's two sheets is not passed
    if status == "failed":
        lead = f'<p class="verdict failed">{NOT_ADOPTED}</p>'
        levers = "".join(
            f"<li><b>{_e(lv.get('id'))}</b> {_e(lv.get('text'))} — {_e(lv.get('cost', ''))}</li>"
            for lv in _list(data.get("levers"))
        )
        lead += f"<h3>诊断与 levers（R-S32-1）</h3><ul>{levers}</ul>"
    else:
        lead = f'<p class="verdict {status}">{_e(STATUS_LINES.get(status, status))}</p>'
    note = data.get("status_note")
    if note:
        lead += f"<p>{_e(note)}</p>"
    credits = cast(Mapping[str, object], data.get("credits", {}))
    blocks: list[str] = []
    for subject in subjects:
        key = str(subject.get("key"))
        parts = [f"<section><h2>{_e(subject.get('name', key))}</h2>"]
        for label, field in (("identity reference", "identity"), ("base-layer pick", "base_layer")):
            value = subject.get(field)
            if value:
                rel = _copy_image(Path(str(value)), out, f"{key}-{field}")
                parts.append(
                    f'<figure class="ref"><img src="{rel}" alt="{label}">'
                    f"<figcaption>{label}</figcaption></figure>"
                )
        pair = subject.get("pair_frame")
        if pair:
            rel = _copy_image(Path(str(pair)), out, f"{key}-pair")
            parts.append(
                f'<figure class="pair"><img src="{rel}" alt="pair">'
                "<figcaption>fused Vera | A | B</figcaption></figure>"
            )
        for outfit in _list(subject.get("outfits")):
            name = str(outfit.get("name"))
            parts.append(
                f"<h3>outfit {_e(name)}: {_e(', '.join(cast(list[str], outfit.get('garments', []))))}</h3>"
            )
            parts.append(_gates_html(outfit))
            parts.append(_frames_html(outfit, out, f"{key}-{name}"))
        parts.append("</section>")
        blocks.append("".join(parts))
    extra = data.get("sections_html", "")
    page = f"""<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>VIS-3D-WARDROBE-P</title>
<style>
body {{ font: 14px/1.5 system-ui, sans-serif; margin: 16px; background: #f4f2ee; color: #222; }}
.grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 12px; }}
figure {{ margin: 0; background: #fff; padding: 6px; border: 4px solid #3a7d44; }}
figure.flagged {{ border-color: #d81414; }}
figure.ref, figure.pair {{ border-color: #888; display: inline-block; }}
figure.ref img {{ max-height: 360px; width: auto; }} img {{ width: 100%; display: block; }}
.verdict {{ font-size: 18px; font-weight: bold; padding: 8px; background: #fff; }}
.verdict.failed {{ color: #d81414; }} .refused {{ color: #d81414; font-weight: bold; }}
table.gates td, table.gates th {{ padding: 2px 6px; border-bottom: 1px solid #ddd; font-size: 12px;
  text-align: left; }}
tr.fail td {{ color: #d81414; }} code {{ font-size: 11px; word-break: break-all; }}
</style></head><body>
<h1>VIS-3D-WARDROBE-P — S32-p wardrobe pilot (route B)</h1>
<p><b>Decision rule.</b> {_e(DECISION_RULE)}</p>
{lead}
<p><b>Credits.</b> Meshy {_e(credits.get("spent", 0))} / {_e(credits.get("cap", 1200))} credits;
OpenAI image calls {_e(credits.get("image_calls", 0))} / {_e(credits.get("image_cap", 70))}.</p>
{extra}
{"".join(blocks)}
</body></html>
"""
    out.mkdir(parents=True, exist_ok=True)
    index = out / "index.html"
    index.write_text(page, encoding="utf-8")
    return index


def prespend(subjects: Sequence[Mapping[str, object]], out: Path) -> Path:
    """The pre-spend page (R-1; step-26 §6.4 gate 1): per subject the identity reference beside the base-layer
    candidates, with the agent's ranking and misses. The operator's pick is recorded in the recipe."""
    blocks: list[str] = []
    for subject in subjects:
        key = str(subject["key"])
        cards: list[str] = []
        identity = _copy_image(Path(str(subject["identity"])), out, f"{key}-identity")
        cards.append(
            f'<figure class="ref"><img src="{identity}"><figcaption>identity reference</figcaption></figure>'
        )
        for k, cand in enumerate(_list(subject.get("candidates")), start=1):
            rel = _copy_image(Path(str(cand["path"])), out, f"{key}-candidate-{k}")
            cards.append(
                f'<figure class="ref"><img src="{rel}"><figcaption><b>{_e(cand.get("file"))}</b> '
                f"rank {_e(cand.get('rank', '-'))}: {_e(cand.get('note', ''))}</figcaption></figure>"
            )
        blocks.append(f"<section><h2>{_e(key)}</h2>{''.join(cards)}</section>")
    page = (
        '<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><title>S32-p pre-spend</title>'
        "<style>body{font:14px/1.5 system-ui,sans-serif;margin:16px}figure{display:inline-block;margin:6px}"
        "img{max-height:420px}</style></head><body>"
        "<h1>S32-p pre-spend：base-layer 参考图（请选择一张或要求重做）</h1>"
        "<p>每个 subject 的 Meshy body 只有在 operator 选定 base-layer 图之后才会运行"
        "（step-26 §6.4 gate 1）。</p>" + "".join(blocks) + "</body></html>"
    )
    out.mkdir(parents=True, exist_ok=True)
    index = out / "prespend.html"
    index.write_text(page, encoding="utf-8")
    return index
