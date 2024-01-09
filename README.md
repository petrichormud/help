# Petrichor Help Repository

This repository contains all the help files for Petrichor MUD.

## To Write a New Help File

### Choose a Unique Slug

Choose a unique slug. For example, "movement", "run", et cetera.
**For now, these slugs must be a single lowercase word, no special characters.**

### Create Metadata

Create a metadata file at `data/metadata`, i.e. `data/metadata/movement.toml`.

This metadata file must contain an `author` key, i.e. `author = "test"`. This must
be the username of an existing player.

A `related` key, like `related = ["run"]`. The `related` field is a list of
slugs of files related to this one.

### Write Your Markdown

Create a `[slug].md` file at `data/help`, i.e. `data/help/movement.md`.
This can be CommonMark compliant but should **not** contain any front matter.
