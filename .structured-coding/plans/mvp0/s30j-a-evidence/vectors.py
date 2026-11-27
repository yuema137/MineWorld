"""Independent integer evaluation of the frozen words specification, not Rust imports.

FNV-1a offset/prime and SplitMix64 output constants are specified in S30-d §5.3.
This records the derivation of fixed test vectors; it does not generate expected
values during cargo tests or search seeds/keys for passing outcomes.
"""
import json

MASK = (1 << 64) - 1


def fnv(text):
    word = 0xCBF29CE484222325
    for byte in text.encode():
        word = ((word ^ byte) * 0x100000001B3) & MASK
    return word


def mix(left, right):
    word = left ^ ((right * 0x9E3779B97F4A7C15) & MASK)
    word = ((word ^ (word >> 30)) * 0xBF58476D1CE4E5B9) & MASK
    word = ((word ^ (word >> 27)) * 0x94D049BB133111EB) & MASK
    return word ^ (word >> 31)


for attribute in ["oracle.choice", "oracle.amount"]:
    for key in ["alpha", "beta", "gamma", "delta"]:
        stream = mix(fnv("independent-contract-v1"), 0)
        word = mix(mix(mix(stream, fnv(attribute)), fnv(key)), 0)
        print(json.dumps({"attribute": attribute, "key": key, "word": word,
                          "ppm": ((word >> 32) * 1_000_000) >> 32}))
