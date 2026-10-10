"""Compression L1 to L3 with retained Event IDs (ARC-59): episodes, chapters and stable facts, decided by
the perceived facts alone, and the summarizers that write their text (§5.5, §5.7).

A model never writes, adds or drops a citation (P4-1): `StructuralSummarizer` writes every summary, and
the optional `embellish` pass (`model_summarizer.py`) may only replace a summary's text with prose.
"""
