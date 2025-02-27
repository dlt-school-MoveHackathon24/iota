# Move

Provides language support for the Move programming language. For information about Move visit the
language [documentation](https://docs.iota.io/concepts/iota-move-concepts).

# How to Install

1. Open a new window in any Visual Studio Code application version 1.61.0 or greater.
2. Open the command palette (`⇧⌘P` on macOS, or use the menu item _View > Command Palette..._) and
   type **Extensions: Install Extensions**. This will open a panel named _Extensions_ in the
   sidebar of your Visual Studio Code window.
3. In the search bar labeled _Search Extensions in Marketplace_, type **Move**. The Move extension
   should appear as one of the option in the list below the search bar. Click **Install**.
4. Open any file that ends in `.move`.

Installation of the extension will also install a platform-specific pre-built move-analyzer binary in
the default directory (see [here](#what-if-i-want-to-use-a-move-analyzer-binary-in-a-different-location)
for information on the location of this directory), overwriting the existing binary if it already exists.
The move-analyzer binary is responsible for the advanced features of this VSCode extension (e.g., go to
definition, type on hover). Please see [Troubleshooting](#troubleshooting) for situations when
the pre-built move-analyzer binary is not available for your platform or if you want to use move-analyzer
binary stored in a different location.

# Troubleshooting

## What if the pre-built move-analyzer binary is not available for my platform?

If you are on Windows, the following answer assumes that your Windows user name is `USER`.

The `move-analyzer` language server is a Rust program which you can install manually provided
that you have Rust development already [installed](https://www.rust-lang.org/tools/install).
This can be done in two steps:

1. Install the move-analyzer installation prerequisites for your platform. They are the same
   as prerequisites for Iota installation - for Linux, macOS and Windows these prerequisites and
   their installation instructions can be found
   [here](https://docs.iota.io/guides/developer/getting-started/iota-install#additional-prerequisites-by-operating-system)
2. Invoke `cargo install --git https://github.com/iotaledger/iota move-analyzer` to install the
   `move-analyzer` language server in your Cargo binary directory, which is typically located
   in the `~/.cargo/bin` (macOS/Linux) or `C:\Users\USER\.cargo\bin` (Windows) directory.
3. Copy the move-analyzer binary to `~/.iota/bin` (macOS/Linux) or `C:\Users\USER\.iota\bin`
   (Windows), which is its default location (create this directory if it does not exist).

## What if I want to use a move-analyzer binary in a different location?

If you are on Windows, the following answer assumes that your Windows user name is `USER`.

If your `move-analyzer` binary is in a different directory than the default one (`~/.iota/bin`
on macOS or Linux, or `C:\Users\USER\.iota\bin` on Windows), you may have the extension look
for the binary at this new location by using VSCode's settings (`⌘,` on macOS, or use the menu
item _Code > Preferences > Settings_). Search for the `move.server.path` workspace setting,
set it to the new location of the `move-analyzer` binary, and restart VSCode.

## What if everything else fails?

Check [Iota Developer Forum](https://forums.iota.io/c/technical-support) to see if the problem
has already been reported and, if not, report it there.

# Features

Here are some of the features of the Move Visual Studio Code extension. To see them, open a
Move source file (a file with a `.move` file extension) and:

- See Move keywords and types highlighted in appropriate colors.
- Comment and un-comment lines of code using the `⌘/` shortcut on macOS (or the menu command _Edit >
  Toggle Line Comment_).
- Place your cursor on a delimiter, such as `<`, `(`, or `{`, and its corresponding delimiter --
  `>`, `)`, or `}` -- will be highlighted.
- As you type, Move keywords will appear as completion suggestions.
- If the opened Move source file is located within a buildable project (a `Move.toml` file can be
  found in one of its parent directories), the following advanced features will also be available:
  - compiler diagnostics
  - go to definition
  - go to type definition
  - go to references
  - type on hover
  - outline view showing symbol tree for Move source files
