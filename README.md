# Petrichor Help Repository

This repository contains all the help files for Petrichor MUD.

## To Write a New Help File

### Choose a Unique Slug

Choose a unique slug. For example, "movement", "run", et cetera.
**For now, these slugs must be a single lowercase word, no special characters.**

### Create Metadata

Create a metadata file at `data/metadata`, i.e. `data/metadata/movement.toml`.

#### Required Keys

`author`:
The helpfile's author. Must be the valid username of an existing player. That
player is not required to have the correct permissions to be marked as an author,
since these helpfiles are already controlled via Git.

`title`:
The title of the help file. Completely appropriate for it to just be
the slug titleized, i.e. "Movement"

`sub`:
The subtitle for the help file, to make it more easily readable in link form.
Include disambiguation as much as possible by including `(Command)` for direct
command help files. There isn't an exhaustive list of these currently, just
try to be clear.

`related`:
A list of slugs of related files. Can (and probably should in most cases)
be two-directional. The destination help file must exist at the slug or
creating the help files will fail.

### Write Your Markdown

Create a `[slug].md` file at `data/help`, i.e. `data/help/movement.md`.
This can be CommonMark compliant but should **not** contain any front matter.

## Examples

Examples can be found at `data/examples`
