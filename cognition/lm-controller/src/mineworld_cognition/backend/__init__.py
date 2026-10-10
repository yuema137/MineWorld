"""Backends behind `ModelBackend`. The HTTP adapter module is never imported here: a `replay` or
`scripted` process must never load a network client (D-P5-5, AP5-6)."""
