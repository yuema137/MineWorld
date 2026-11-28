"""Read nextest's structured failure identities; exit status remains the verdict authority."""

from pathlib import Path
from xml.etree import ElementTree


def failures(path: Path) -> list[str]:
    root = ElementTree.parse(path).getroot()
    if root.tag != "testsuites":
        raise ValueError(f"{path}: expected testsuites")
    found = []
    count = 0
    for suite in root.findall("testsuite"):
        binary = suite.attrib["name"]
        for case in suite.findall("testcase"):
            count += 1
            name = case.attrib["name"]
            for tag in ("failure", "error"):
                if (problem := case.find(tag)) is not None:
                    found.append(f"`{binary}`: `{name}` ({problem.get('type', tag)})")
    if not count:
        raise ValueError(f"{path}: no test records")
    return found
