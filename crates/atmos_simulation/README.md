# Space Station 2197 Atmospherics

## Introduction

This is a tile-based atmos implementation, simple but unsatisfactory.
For this reason I am calling it `CLANKmos` (not an acronym for anything, just that it is clanky as hell.).
Hopefully this can lay the groundwork, until a better (but reasonable) zone-based solution arrives.

> [!WARNING]
> Currently no value is cached. (e.g. temperature() is called multiple times needlessly).
> Lots of repeated code, particularly in delta building.
> Expect all of this this to change **very** *soon*.

If you need to get a good understanding of gas physics, I unironically reccomend Wikipedia articles, 
starting with the [Ideal Gas Law](https://en.wikipedia.org/wiki/Ideal_gas_law).

The simulation utilizes fixed-sized arrays indexed by `GasId` to store molar counts and partial pressures for cache efficiency and pipeline consistency.
My intention was for this structure to ensure predictable processing. Drawbacks are that it increases the memory footprint and forces the system to perform redundant calculations on negligible values that could otherwise be skipped. 
And talking about redudant calculations, tiles are processed in 8x8 chunks! This means that even though a subregion of the chunk might have actual transfers going in, this will force all tiles in a chunk to be processed!

And the result of all of this?
~70 μs/chunk processing cost. Without reactions or hotspots. With most of it wasted on interchunk deltas.
This is a lot and not that efficient.


## Primitives

### `GasId`
All gases are attributed a GasId that is unique to them and used for indexing them in the multiple arrays.
### `GasList`
All gas definitions are contained here, serving as immutable lookup for gas details.
### `GasMixture`
This is what contains the gases. 
It should always be interpreted as a homogeneous composition of multiple gases (given by mole count),
with internal energy and a fixed volume.
### `Reaction`
Reactions currently are expressed as simple linear equations, where they transform gases into other gases,
given conditions are met.
### `MixtureTemplate`
Predefined mixtures that can be applied to GasMixtures. Good for setting known mixtures such as space station air or cold room airs.

## Environmental Process
### `Active`
Marker component for chunks that are active.
An active chunk
### `Mixtures`
### `ChunkDeltas`
### `MixturesQuery`

For more information do check the code! It is implemented in the `engine` module.

