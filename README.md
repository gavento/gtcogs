# GT Cogs
[![Build Status](https://travis-ci.org/gavento/gtcogs.svg?branch=master)](https://travis-ci.org/gavento/gtcogs)

Sketch of a [game theory](https://en.wikipedia.org/wiki/Game_theory) framework and API in [Rust](https://www.rust-lang.org/).

Currently supports:
* Game interface with history, obsevations, active player, strategies etc.
* Goofspiel implementation (with hidden opponent moves)
* Outer sampling MCCFR implementation (very fast!)
* Generic tree game (currently limited to copying an existing game)

This repo is currently very experimental. I started it as an exploration* of the right API for game theory
framework in Rust to be subsequently reimplemented similarly in [GameGym](https://github.com/gavento/gamegym).
It *could* be a base of a GT lib in Rust but currently there are no plans in this direction.

*) Have you also noticed that drafting APIs in a very strict language forces you to think through some
decisions you would notice only much later in e.g. Python?
