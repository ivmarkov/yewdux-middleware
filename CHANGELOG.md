# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2024-02-01
* Breaking change: update to `yewdux 0.10` which introduces the notion of `Context`. As a result, the following changes were necessary, which roughly follow the changes w.r.t. `Context` in `yewdux`:
  * New notion of middleware context: `MiddlewareContext`
  * New component - `YewduxMiddlewareRoot`
  * `Middleware` and `MiddlewareDispatch` now take an extra parameter of type `&MiddlewareContext`
  * top level dispatch functions `void`, `dispatch`, `get` moved as members of `MiddlewareContext`
* Breaking change: all crate sub-modules are now private. Their types re-exported at the crate top-level
