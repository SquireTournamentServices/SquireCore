[![Crates.io](https://img.shields.io/crates/v/squire_core.svg)](https://crates.io/crates/squire_core)
[![Documentation](https://docs.rs/squire_core/badge.svg)](https://docs.rs/squire_core/)
![GitHub Workflows](https://github.com/MonarchDevelopment/SquireCore/actions/workflows/ci.yml/badge.svg)
[![Coverage Status](https://codecov.io/gh/MonarchDevelopment/SquireCore/branch/rust-port/graph/badge.svg)](https://codecov.io/gh/MonarchDevelopment/SquireCore)
![Maintenance](https://img.shields.io/badge/Maintenance-Actively%20Developed-brightgreen.svg)

## Overview
This library contains the backend library for tournament management used by [SquireBot](https://github.com/MonarchDevelopment/SquireBot).
The goal is to allow for a series of pairing and scoring systems to be
combined in a dynamic way, allowing for on-the-fly, highly customizable
tournament structures.

## Current State
SquireCore is still very much in development.
SquireBot implemented many of the core functionalities in Python.
The features in this library are highly inspired by that implementation.

## Future Plans
Eventually, this project will be a fully-fledged backend service, not
just a library.
Due to potential time constraints, the library portion of this project
will be completed first.
After which time, work on the actual service will start.
