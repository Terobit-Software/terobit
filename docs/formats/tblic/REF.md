# terobit license - .tblic

proves that a user owns a realm. it lets a client start a `Licensed` realm with no network
connection.

one license per user per realm. the platform issues it on purchase, and the client
downloads it whenever it's online and keeps it with that user's data. it isn't part of
any pack, so a pack is identical for everyone who owns it and an update never needs new
licenses.

current version: v1 (draft)
oldest supported: v1

## versions

- [v1](v1.md)

## offline check

when the manifest's `access` is `Licensed` (`Free` skips this):

1. take the user id of the last user who logged in on this machine
2. find a license with that `user_id` and the manifest's `realm_id`. none: refuse to run
3. verify it (see [v1](v1.md#verifying))
4. passes: run

licenses don't expire. a refund or a removed purchase is handled the next time the client
is online, when the platform tells it to delete the license.

## what this can and can't stop

it stops the easy cases: copying a game to another account or machine doesn't copy the
right to play it.

the "last logged in user" record should itself be signed by the platform at login, so
editing a local file doesn't switch users. that's part of the account system, not a format
here yet.

no offline check can stop someone who copies a whole machine's data or modifies the
client. online play doesn't rely on this file: servers check ownership through the
platform.

## changelog

- v1: initial version
