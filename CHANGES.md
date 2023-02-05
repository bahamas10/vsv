vsv Changes
===========

Not Yet Released
----------------

(nothing yet)

`v2.0.0`
--------

- Rewritten to Rust from bash.  Archived code is available here: https://github.com/bahamas10/bash-vsv/blob/master/CHANGES.md

`v1.3.5`
--------

- [#16](https://github.com/bahamas10/vsv/pull/16) loosen-up defensive service name validation logic to allow "@" chars

`v1.3.4`
--------

- [#8](https://github.com/bahamas10/vsv/issues/8) better detect terminals with color support

`v1.3.3`
--------

- [#4](https://github.com/bahamas10/vsv/pull/4) Truncate command output to not overflow 17 characters (PR by @zdykstra)
- [#6](https://github.com/bahamas10/vsv/issues/6) race condition when checking if a service is disabled

`v1.3.2`
--------

- Add license file (MIT)

`v1.3.1`
--------

- Update manpage

`v1.3.0`
--------

- [#1](https://github.com/bahamas10/vsv/pull/1) fix a typo (PR by @pltrz)
- [#2](https://github.com/bahamas10/vsv/pull/2) Add support to enable/disable service (issue by @illiliti)

`v1.2.1`
--------

- Add manpage

`v1.2.0`
--------

- Add `-l` to status to show log processes
- Add `COMMAND` to output (first arg of `cmdline`)

`v1.1.0`
--------

- Add `-V` to print version number

`v1.0.0`
--------

- Initial Commit
