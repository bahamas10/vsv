VSV 8 "FEB 2022" "System Manager's Utilities"
=============================================

NAME
----

`vsv` - manage and view runit services

SYNOPSIS
--------

`vsv [OPTIONS] [SUBCOMMAND] [<ARGS>]`

`vsv [-u] [-d <dir>] [-h] [-t] [SUBCOMMAND] [...]`

DESCRIPTION
-----------

`vsv` is a wrapper for the `sv` command that can be used to query and manage
services under runit. It was made specifically for Void Linux but should
theoretically work on any system using runit to manage services.

OPTIONS
-------

`-c <yes|no|auto>`
  Enable/disable color output, defaults to auto.

`-d` *dir*
  Directory to look into, defaults to env `SVDIR` or `/var/service` if unset.

`-h`
  Print this message and exit.

`-l`
  Show log processes, this is a shortcut for `vsv status -l`.

`-t`
  Tree view, this is a shortcut for `vsv status -t`.

`-u`
  User mode, this is a shortcut for `vsv -d ~/runit/service`.

`-v`
  Increase verbosity.

`-V`
  Print the version number and exit.

ENVIRONMENT
-----------

`SVDIR`
  The directory to use, passed to the `sv` command, can be overridden with `-d
  <dir>`.

`PROC_DIR`
  A Linux procfs directory to use for command name lookups, defaults to `/proc`.

`SV_PROG`
  The command to use for any "external" subcommand given to `vsv`, defaults to
  `sv`.

`PSTREE_PROG`
  The command to use to get a process tree for a given pid, defaults to
  `pstree`.

`NO_COLOR`
  Set this environmental variable to disable color output.

SUBCOMMANDS
-----------

`status`

`vsv status [-lt] [filter]`

Default subcommand, show process status

`-t`
  Enables tree mode (process tree)

`-l`
  Enables log mode (show log processes)

`filter`
  An optional string to match service names against

Any other subcommand gets passed directly to the `sv` command, see `sv(1)` for
the full list of subcommands and information about what each does specifically.
Common subcommands:

`start <service>`

  Start the service

`stop <service>`

  Stop the service

`restart <service>`

  Restart the service

`reload <service>`

  Reload the service (send `SIGHUP`)

`enable <service>`

 Enable the service (remove the "down" file, does not start service)

`disable <service>`

 Disable the service (create the "down" file, does not stop service)

EXAMPLES
--------

`vsv`

  Show service status in `/var/service`

`vsv status`

  Same as above

`vsv -t`

  Show service status + `pstree` output

`vsv status -t`

  Same as above

`vsv status tty`

  Show service status for any service that matches `tty`

`vsv check uuidd`

  Check the uuidd svc, wrapper for `sv check uuidd`

`vsv restart sshd`

  Restart sshd, wrapper for `sv restart sshd`

`vsv -u`

  Show service status in `~/runit/service`

`vsv -u restart ssh-agent`

  Restart ssh-agent in `~/runit/service/ssh-agent`

BUGS
----

https://github.com/bahamas10/rust-vsv

AUTHOR
------

`Dave Eddy <bahamas10> <dave@daveeddy.com> (https://www.daveeddy.com)`

SEE ALSO
--------

sv(8), runsvdir(8)

LICENSE
-------

MIT License
