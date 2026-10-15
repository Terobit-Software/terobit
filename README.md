## Commands

### cargo rcl
Run client.

|args|desc|
|----|----|
|--final|builds client as final, using release profile instead of dev|
|--strip-defaults|removes default developer flags from client. removes --dev, and --pack-file at '.' which needs to be updated|

### cargo pkgen
Build .TBPK file from manifest.

|args|desc|
|--|--|
|--manifest|the path to the manifest.toml|
|--out|the path to the out directory for the packages|
|--targets|client, server, or both|


### cargo regen
Build examples for generating test and example packages.
|args|desc|
|--|--|
|--type|xsmall, small, large, xlarge, mix, xmix|
|--auto|runs pkgen automatically after generating|
|--count|count of files for both sides, can be "random"|
|--max|if random is chosen for --count, max amount of files|
|--min|if random is chosen for --count, min amount of files|
|--entropy|the level of entropy in the output, low, med, high|
