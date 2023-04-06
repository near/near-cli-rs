# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2023-04-06

* Renamed `near-cli` binary to `near` as it has special handlers for the commands of near CLI JS, and can be used as a replacement to near CLI JS that will guide users on how to use the new commands when they type the old commands.
* Improved continuous integration pipelines to streamline releases (each push to `master` branch will trigger a pipeline that will create a PR suggesting to cut a new release, and once the version is ticked, crate will be published and tagged, and then binary release pipeline will kick in)
