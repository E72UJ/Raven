<p align="center">
  <a href="https://github.com/ruffle-rs/h263-rs/actions">
      <img alt="Rust Build Status" src="https://img.shields.io/github/actions/workflow/status/ruffle-rs/h263-rs/test_rust.yml?label=Rust%20Build&logo=github&branch=master" />
  </a>
  <a href="https://discord.gg/ruffle">
      <img src="https://img.shields.io/discord/610531541889581066" alt="Ruffle Discord">
  </a>
  <br>
  <strong><a href="https://ruffle.rs">website</a> | <a href="https://ruffle.rs/demo">demo</a> | <a href="https://github.com/ruffle-rs/h263-rs/releases">nightly builds</a> | <a href="https://github.com/ruffle-rs/h263-rs/wiki">wiki</a></strong>
</p>

# h263-rs

h263-rs is a pure-Rust implementation of ITU-T Recommendation H.263 (2005/08), a video codec commonly used in early VoIP telephony and multimedia systems including Sorenson Spark and Adobe Flash Player. It is used primarily in Ruffle to provide H.263 video decoding capability.

## Project status

h263-rs correctly decodes most Sorenson-flavor video streams. No attempt has yet been made to test other flavors of H.263, or any of the additional features in later versions of H.263.

There is currently no support for encoding H.263 video of any flavor.
## Using h263-rs

Currently, this only ships as a library, which must be integrated in another project to play video.

## Building from source

[Follow the official guide](https://www.rust-lang.org/tools/install) to install Rust for your platform.

## Structure

- `h263` contains the core codec library
- `yuv` contains BT.601 YUV colorspace conversions needed for decoding H.263 video

## Sponsors

This project is maintained by the developers of Ruffle. You can support the development of Ruffle via [GitHub Sponsors](https://github.com/sponsors/ruffle-rs). Your sponsorship will help to ensure the accessibility of Flash content for the future. Thank you!

Sincere thanks to the diamond level sponsors of Ruffle:

<p align="center">
  <a href="https://www.newgrounds.com">
    <img src="https://ruffle.rs/assets/sponsors/newgrounds.png" alt="Newgrounds.com">
  </a>
  <a href="https://www.cpmstar.com">
    <img src="https://ruffle.rs/assets/sponsors/cpmstar.png" alt="CPMStar">
  </a>
  <a href="https://deepnight.net">
    <img src="https://ruffle.rs/assets/sponsors/deepnight.png" alt="Sébastien Bénard">
  </a>
  <a href="https://www.crazygames.com">
    <img src="https://ruffle.rs/assets/sponsors/crazygames.png" alt="Crazy Games">
  </a>
  <a href="https://www.coolmathgames.com">
    <img src="https://ruffle.rs/assets/sponsors/coolmathgames.png" alt="Cool Math Games">
  </a>
  <a href="https://www.nytimes.com/">
    <img src="https://ruffle.rs/assets/sponsors/nyt.png" alt="The New York Times">
  </a>
  <a href="https://www.armorgames.com/">
    <img src="https://ruffle.rs/assets/sponsors/armorgames.png" alt="Armor Games">
  </a>
  <a href="https://www.ondaeduca.com/">
    <img src="https://ruffle.rs/assets/sponsors/ondaeduca.png" alt="Onda Educa">
  </a>
  <a href="https://www.twoplayergames.org/">
    <img src="https://ruffle.rs/assets/sponsors/twoplayergames.png" alt="TwoPlayerGames.org">
  </a>
  <a href="https://www.wowgame.jp/">
    <img src="https://ruffle.rs/assets/sponsors/wowgame.png" alt="wowgame.jp">
  </a>
  <a href="http://kupogames.com/">
    <img src="https://ruffle.rs/assets/sponsors/mattroszak.png" alt="Matt Roszak">
  </a>
</p>

## License

h263-rs is licensed under either of

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT License (http://opensource.org/licenses/MIT)

at your option.

h263-rs depends on third-party libraries under compatible licenses. See [LICENSE.md](LICENSE.md) for full information.

### Contribution

h263-rs welcomes contribution from everyone. See [CONTRIBUTING.md](CONTRIBUTING.md) for help getting started.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

The entire h263-rs community, including the chat room and GitHub project, is expected to abide by the [Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct) that the Rust project itself follows.
