# what_stream

simply provide this a query to search for and it'll list programming streams that match

a query is a space separated list of words.

e.g. `"c++ rust gamedev"`

the `query` will match words in the stream title. the query is case-insensitive. so `opengl` will match `OpenGL`

to filter for a spoken language, use the `-l`,`--language` flag. this can be used in succession.

the format for these language filters are [`ISO-639-1`](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes). e.g. 'en' or 'de'

## requirements

the following environment variables must be set while building:

`WHAT_STREAM_CLIENT_ID`

- this your Twitch API Client-ID

`WHAT_STREAM_CLIENT_SECRET`

- this your Twitch API client secret

### how to get these tokens, a primer

1. to get these tokens, visit [https://dev.twitch.tv/console](https://dev.twitch.tv/console)
1. then go to `applications`
1. then `register your application`
   1. enter a name
   1. the redirect url can be `http://localhost`
   1. category can be anything, but you should probably just choose `Analytics Tool`
1. set the env var `WHAT_STREAM_CLIENT_ID` to the value of `Client ID`
1. create a new secret by clicking `New Secret`
   1. set the env var `WHAT_STREAM_CLIENT_SECRET` to that value

## example

this'll look for streams with `c++` or `python` or `rust` in their title.

`> what_stream rust c# go`

```
┌── rust
├ [EN] https://twitch.tv/museun
├ random projects in Rust
└ started 1 hour 10 minutes ago, 9 watching

┌── c#
├ [PL] https://twitch.tv/mr_komugiko
├ Pisanie prostej gierki ;d / MMO / C# / Unity
├ started 35 minutes ago, 1 watching
│
├ [EN] https://twitch.tv/x_coding
├ Playing with some C#. Bug fixes, git and so on - !help
├ started 2 hours 3 minutes ago, 5 watching
│
├ [EN] https://twitch.tv/oxcanteven
├ Twitch Audio Channel Point Redemptions | C#, .Net Core, Avalonia
└ started 3 hours 18 minutes ago, 7 watching
```

**note** if a query does not match, it will not be displayed.

also, try using one of the `--style` options

## usage

```
what_stream 0.3.0

USAGE:
    what_stream [flags] [query ..]

FLAGS:
    -h, --help                 show the help message
    -v, --version              show the current version
    -l, --language <language>  filter to this specific language
    -s, --sort <col,dir?>      sort by <col> in the optional <dir>
    -t, --style <style>        what type of rendering style to use
    -j, --json                 dumps the results as json

SORTING:
    available columns:
     - name (the default)
     - viewers
     - uptime

    available directions:
     - descending (the default)
     - desc (shorthand)

     - ascending
     - asc (shorthand)

QUERY:
    query is a space separated list of 'tags' to filter by.
    the tags are case-insensitive and will match 'words' in the stream title
    e.g. 'opengl' will match 'OpenGL' in a title: "making a game with OpenGL"

STYLES:
    - box (the default)
    - fancy
    - none

NOTES:
    the --language flag can be used multiple times. e.g. `-l en -l pt -l de`
    - a language value is in the form of `ISO 639-1`

    if `NO_COLORS` is set, the colors are disabled
```

## license

[0BSD](./LICENSE.txt)
