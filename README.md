[![Crates.io](https://img.shields.io/crates/v/squire_core.svg)](https://crates.io/crates/squire_core)
[![Documentation](https://docs.rs/squire_core/badge.svg)](https://docs.rs/squire_core/)
![GitHub Workflows](https://github.com/MonarchDevelopment/SquireCore/actions/workflows/ci.yml/badge.svg)
[![Coverage Status](https://codecov.io/gh/MonarchDevelopment/SquireCore/branch/main/graph/badge.svg)](https://codecov.io/gh/MonarchDevelopment/SquireCore)
![Maintenance](https://img.shields.io/badge/Maintenance-Actively%20Developed-brightgreen.svg)

## Overview
This repo contains the core functionalities of the Squire Touranment Services.
STS aims to provide highly flexible tools for organizing and running Magic: the Gathering tournaments.
This includes pairing algorithms that can handle an arbitrary number of players, highly customizable scoring systems and scoring methods, and allowing for on-the-fly adjustments of all settings.

Here you will find three primary subdirectories that all Squire services utitilize.

Starting off, the tournament model is defined in `squire_lib`.
This is the very core of STS.
Our tournament model where we implement all of the aforementioned featured.
This library is used by every Squire client to ensure a uniform tournament expirence everywhere.

Next, the backend server can be found in `squire_core`.
This connects most clients together and helps them manage and syncronize their tournament data.

Lastly, there is the `squire_sdk`.
The SDK is a general toolkit used by both the backends (namely `squire_core`) and all clients.
It contains a model for syncronizing tournaments between clients and a backend, a generic client for communicating with a backend and that can be customized to fit the client's needs, as well as API data models used for sending and recieving requests from a backend.


## How It Works
All Squire services work under a local-first model.
This means that the status of the SquireCore backend will have limited effect on a client's ability to run and manage a tournament.

To do this, every service (client or backend) needs to share the same implementation of the tournament model and tournament sync procedure.
This is the job of `squire_lib` and `squire_sdk`, respectively.

This is why everything in this repo is written in Rust.
It is a langauge that can be distributed very easily across most platforms, can be run in the browser via WebAssembly (WASM), and is light-wieght and reliable.


## Clients and Services
Currently, there are two ready Squire services outside of this repo:
 - [SquireDesktop](https://github.com/MonarchDevelopment/SquireDesktop)
 - [SquireBot](https://github.com/MonarchDevelopment/SquireBot)

SquireDesktop is a native desktop application that lets you create and manage tournaments from your desktop.
Likely any Squire client, it can backup all tournament data to SquireCore in order to make it publically accessible.

SquireBot is a Discord bot that can be used as a full Squire client, like the desktop.
However, its primary usecase is to notify players about the status of the tournament, such as being paired for a new match, the time left in the round, and the current standings.

There are plans for many more services in the future.
To learn more, see our [future plans](##Current-and-Future-State).


## Current and Future State
This repo (and STS as a whole) are still growing rapidly.
Currently, everything in this repo is focused around proving the functionality of the tournament model; however, SquireCore (and therefore SquireSDK) will soon support many more things:
 - Searchable and queriable APIs for finding tournaments
 - Users, so players can track their involvement across tournaments
 - Organizations, to make finding tournaments easier for players and creating/managing tournament easier for organizers

 We also have plans for the following services, all of which will be integrated together:
  - SquireWeb, our web frontend for all-things Squire
  - SquireMobile, Android and Apple mobile apps for players to manage participation in tournaments
  - SquireJudge, mobile apps designed for judges and TOs to track and manage parts of a tournament
  - SquireFall, a public database for searching and querying past tournament data (akin a tournament version of [Scryfall]([Scryfall](https://scryfall.com/advanced)))
