# Space Station 2197
### Space Station 13 reinvented in 3D.

- [Website](https://www.spacestation2197.com)
- [Documentation](https://docs.spacestation2197.com)
- [Discord](https://discord.com/channels/691052431525675048/1495401185937461298) - Join [Bevy's discord](https://discord.gg/9TsWQARU6k) first!

## Development

This project is in very early development. So early, in fact, that this README is quite empty!

TL;DR:
Bevy game engine underneath.
Lua for content package.
TBD runtime scripting engine.

## Contributing

*Human-made* contributions are welcome from everyone!
Start off by joining the [Discord](https://discord.gg/MsV9Jg42XQ) or picking one of the [Issues](https://github.com/malfuu/spacestation2197/issues).

## Crates

| Crate              | Description                                      |
|--------------------|--------------------------------------------------|
| atmos              | Atmospheric simulation code                      |
| common             | Common components + messages, should be deleted soon|
| tile_grid          | Tilemap grid implementation + grid relations     |
| scripting          | Prototype scripting w/ Lua         |
| shared             | Shared code between client and server, mostly definitions |
| client             | Client executable                                |
| server             | Authoritative/Logic code library + Server executable code |

## License

#### Code
All of the source code with the exception of the `assets/` folder is licensed under the **Mozilla Public License 2.0**, under the following notice:

> This Source Code Form is subject to the terms of the Mozilla Public
> License, v. 2.0. If a copy of the MPL was not distributed with this
> file, You can obtain one at https://mozilla.org/MPL/2.0/.
>
> This Source Code Form is "Incompatible With Secondary Licenses", as
> defined by the Mozilla Public License, v. 2.0.

See [LICENSE](LICENSE) file for more details.

#### Assets
All assets under the ```assets/``` folder fall under the [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) license unless stated otherwise in accompanying `attribution.txt` files. See [LICENSE-ASSETS](LICENSE-ASSETS) file for more details.

#### Contributions
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed as above, without any additional terms or conditions.
