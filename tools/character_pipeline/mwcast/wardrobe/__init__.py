"""The S32-p wardrobe pilot's driver: route B bodies, the garment pipeline, the wardrobe gates (MIT, stdlib).

The specification is `pr-s32p-wardrobe-pilot.md` §4 and `tools/character_pipeline/PIPELINE.md` §§16–17. No
module here imports `bpy` or MPFB2 (I-p-10); the Blender stages under `blender/mpfb/` are subprocesses only.
"""
