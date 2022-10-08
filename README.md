## Cantaloupe - 1.0.0
##### Jonathan Vasquez (fearedbliss)

## Description

A simple backup replication tool for OpenZFS.

## Usage

To start using the application, all you need to do is run:

**`./cantaloupe <backup pool> <label> <datasets> ...`**

**Example:**

**`./cantaloupe backup CHECKPOINT tank/os/main tank/var/log`**

### Notes

- The user running this application needs to have permissions to use the
  **`zpool`** and **`zfs`** utilities, and needs to have permission to
  write to the disks you wish to replicate into. If you just want to
  preview what will happen, you can perform a dry run (**`-n`**) which
  only requires access to the zfs utilities.
- The **`zpool`** and **`zfs`** utilities need to be in your **`PATH`**.
- You can specify multiple datasets that are located in different pools
  in your datasets list. However, none of them may be in the same pool as
  the backup pool.

## Format

Cantaloupe uses the same snapshot format as [Honeydew](https://github.com/fearedbliss/Honeydew):

**`YYYY-mm-dd-HHMM-ss-LABEL`** => **`2022-09-01-1234-56-ANIMALS`**

The following script will take a snapshot in the correct format:

```
#!/bin/sh

POOL="tank"
DATE="$(date +%F-%H%M-%S)"
TAG="ANIMALS"
SNAPSHOT_NAME="${DATE}-${TAG}"

zfs snapshot "${POOL}@${SNAPSHOT_NAME}"
```

Any snapshots that are not in this format will be gracefully skipped.

## Options

```
Usage: cantaloupe [OPTIONS] <BACKUP_POOL> <LABEL> <DATASETS>...

Arguments:
  <BACKUP_POOL>
  <LABEL>
  <DATASETS>...

Options:
  -n, --dry-run  Performs a dry run. Does not require root privileges.
  -h, --help     Print help information
  -V, --version  Print version information
```

## Build

The easiest way to build the project is to have **`cargo`** installed and run:
**`cargo build --release`**.

## License

Released under the **[Simplified BSD License](LICENSE)**.

## Contributions

Before opening a PR, please make sure the code is properly formatted and all
tests are passing. You can do this by running: **`cargo fmt`** and
**`cargo test`** respectively.
