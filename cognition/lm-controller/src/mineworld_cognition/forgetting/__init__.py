"""The bundled retention policies (pr-fg-b §5.9): ordinary plug-ins of the entry-point group
`mineworld.forgetting`, none privileged (FGb-4).

They sit outside `memory/` so that they are packaged and imported exactly as a user's policy would be:
each imports only the public names of `mineworld_cognition.memory.retention`, Pydantic for its
`Parameters`, and the standard library's typing and `hashlib`; none reads a clock, a file or a store
(FGb-3, AFGb-9). An assisted policy (`lm-summarise`, pr-fg-c) asks a model only through the host's
`SummaryGateway`. The host finds them through their entry points, never by import.
"""
