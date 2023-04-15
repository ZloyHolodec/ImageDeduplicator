# Simple and fast tool for image de duplication.

This tool can find image duplicates and remove them. It may found duplicates even for resized or slightly changed images.

**This application is raw at this point. It mostly proof of concept than real application for wide use. It may have some bugs and provided “as is”**


## How to execute

Install GTK4 libraries:
> sudo apt install libgtk-4-common

Create sqlite db:
> touch database.sqlite

Compile:

> cargo run

## How to use
1. Click on `Add folders` to choose folders for search images.
2. Click `Scan` and wait until all images be found and their hashes are calculated.
3. After that candidates to duplicated images should appear on UI. You can decide which image should be deleted or decide to save both by clicking on bottom buttons.