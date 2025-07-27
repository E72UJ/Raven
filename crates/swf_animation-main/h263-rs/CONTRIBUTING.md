# Contributing to h263-rs

🎉 Thanks for your interest in h263-rs! Contributions of all kinds are welcome.

This document serves as a general guide for contributing to h263-rs. Follow your best judgement in following these guidelines.

## Table of Contents

 * [Getting Started](#getting-started)
 * [Ways to Contribute](#ways-to-contribute)
   * [Test your favorite video content](#test-your-favorite-video-content)
   * [Improve documentation](#improve-documentation)
   * [Fix interesting issues](#fix-interesting-issues)
   * [Implement missing H.263 functionality](#implement-missing-h-263-functionality)
 * [Reporting Bugs](#reporting-bugs)
 * [Debugging ActionScript Content](#debugging-actionscript-content)
 * [Code Guidelines](#code-guidelines)
 * [Commit Message Guidelines](#commit-message-guidelines)
 * [Pull Requests](#pull-requests)

## Getting Started

The [h263-rs wiki](https://github.com/ruffle-rs/h263-rs/wiki) is a great way to familiarize yourself with the project. It contains info on how to build h263-rs, using h263-rs, and links to helpful documentation about the H.263 video format.

Feel free to ask questions in our [Discord server](https://discord.gg/J8hgCQN).

## Ways to Contribute

We love new contributors! You can contribute to h263-rs in several ways:

### Test your favorite video content

Currently, the easiest way to test h263-rs is by loading Flash movies and games that use H.263 video into Ruffle. More testing pathways will be present as the project matures.

### Improve documentation

Improving documentation is a great way to learn the codebase. Adding documentation to both the wiki and the code eases the learning curve for both end users and new contributors.

For documentation in the code, we follow the [rustdoc](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments) guidelines.

### Fix interesting issues

Try your hand at fixing [issues that are interesting to you](https://github.com/ruffle-rs/h263-rs/issues). Follow the instructions on [building h263-rs](https://github.com/ruffle-rs/h263-rs/wiki/Building-h263-rs), familiarize yourself with the [project layout](https://github.com/ruffle-rs/h263-rs/wiki/Project-Layout), and use (TODO: what debug resources are there for H.263 video streams?) to help debug the issue.

You can also ask for mentoring on our [Discord server](https://discord.gg/J8hgCQN).

### Implement missing H.263 functionality

h263-rs is a young project, and there is still much H.263 functionality that is unimplemented. Check for the ["unimplemented"](https://github.com/ruffle-rs/h263-rs/issues?q=is%3Aissue+is%3Aopen+label%3Aunimplemented) in issues.

## Reporting bugs

[Issue reports and feature requests](https://github.com/ruffle-rs/h263-rs/issues) are encouraged, and are a great way to measure our progress!

When filing an issue, if possible, please include:

 * A clear description of the problem
 * The platform you are testing on (web, desktop, OS)
 * A link/attachment to the video stream demonstrating the issue, if possible
 * Screenshots if the issue is a visible problem
    * Bonus points for including the correct output from another video decoder

These types of focused issues are helpful:

 * Tracking issues for specific H.263 features (non-base-profile features, other flavors of H.263)
 * Bug reports for specific videos with decoding issues (macroblock artifacts, etc.)

The project is still in the early stages, so many H.263 features outside of Sorenson Spark are unimplemented and not yet expected to work. Please avoid filing generic issues such as:

 * A "this video file doesn't work at all" report (what about it doesn't work?)
 * Duplicate issues for each video using an unimplemented feature
 * Asking for dates when a feature will be implemented

## Code Guidelines

h263-rs is built using the latest stable version of the Rust compiler. Nightly and unstable features should be avoided.

The Rust code in h263-rs strives to be idiomatic. The Rust compiler should emit no warnings when building the project. Additionally, all code should be formatted using [`rustfmt`](https://github.com/rust-lang/rustfmt) and linted using [`clippy`](https://github.com/rust-lang/rust-clippy). You can install these tools using `rustup`:

```sh
rustup component add rustfmt
rustup component add clippy
```

You can auto-format your changes with `rustfmt`:

```sh
cargo fmt --all
```

and you can run the clippy lints:

```sh
cargo clippy --all --tests
```

Specific warnings and clippy lints can be allowed when appropriate using attributes, such as:

```rs
#[allow(clippy::float_cmp)]
```

### Test Guidelines

Heavily algorithmic code may benefit from unit tests in Rust: create a module `mod tests` conditionally compiled with `#[cfg(test)]`, and add your tests in there.

## Commit Message Guidelines

Here is a sample commit message:

```
h263: Implement B-Picture decoding
```

 * If applicable, prefix the first line with a tag indicating the relevant area of changes:
   * `h263:`
   * `yuv:`
   * `docs:`
   * `chore:`
   * `tests:`
 * Capitalize the first letter following the tag
 * Limit line length to 72 characters
 * Use the present tense and imperative mood ("fix", not "fixed" nor "fixes")
 * Reference any PRs or issues in the first line
 * Use keywords to close/address issues when applicable ("close #23")
 * Write more detailed info on following lines when applicable

## Pull Requests

Pull requests are the primary way to contribute code to h263-rs. Pull requests should be made against the latest `master` branch. Your pull request should not contain merges; you should always rebase when bringing the latest changes into your branch from the `master` branch. If there are merge conflicts, or if your commit history is messy, please rebase onto the latest master. [`git rebase -i`](https://thoughtbot.com/blog/git-interactive-rebase-squash-amend-rewriting-history#interactive-rebase) is a great way to clean up your pull request.

When you make a pull request, our [CI](https://circleci.com/gh/ruffle-rs/h263-rs) will build your changes and run them through all tests and style checks. All of these tests should pass before your pull request can be accepted.

One of [our regular contributors](https://github.com/orgs/ruffle-rs/people) will review your changes and try their best to helpfully suggest any changes. If all goes well, your PR should be merged without much delay. We use both standard merge commits and fast-forward merges depending on the size of the changes. Thanks for your contribution!
