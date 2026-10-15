# mineworld-motivated

A decision model for MineWorld that needs no language model. It scores what a person can do, and its
daily rhythm is calibrated from published time-use data.

The calibration data is Table A-3 of the **American Time Use Survey** (U.S. Bureau of Labor Statistics,
2025 annual averages). The table is in the public domain. Credit: *Source: U.S. Bureau of Labor
Statistics, American Time Use Survey.* See [`data/NOTICE`](data/NOTICE) for the details.

This crate is in progress. So far it holds the committed table, its checks, and the curves derived from
it.
