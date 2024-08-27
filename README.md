# zed-metals
A Scala extension for Zed (powered by Metals)

## Pre-requisites
* [Coursier](https://get-coursier.io/)
* Install Metals: `cs install metals`

Note: You need to have the path to `metals` exported at shell init (e.g. by an entry in `~/.bashrc`), as `zed` does not currently seem to pick up exported environment variables when started from a terminal. So it's not enough to `export PATH="$PATH:~/.local/share/coursier/bin"` in a shell and run `zed` from there. It will fail to start Metals in that case (and will not say so in the LSP log; but nothing but syntax highlighting will work then).

## Configuration

You can set [Metals user configuration settings](https://scalameta.org/metals/docs/integrations/new-editor/#metals-user-configuration)
in your zed settings.json in `lsp.metals.settings`. For example, to enable displaying type annotations for inferred types
as inlay hints:

``` json
{
  "lsp": {
    "metals": {
      "settings": {
        "inlayHints": {
          "inferredTypes": {
            "enable": true
          }
        }
      }
    }
  }
}
```

You can also set [Metals initialization options](https://scalameta.org/metals/docs/integrations/new-editor/#initializationoptions) and
[Metals server properties](https://scalameta.org/metals/docs/integrations/new-editor#metals-server-properties) in your zed settings.json
in `lsp.metals.binary.arguments` and `lsp.metals.initialization_options`, respectively.

For example, to [enable HTTP server](https://scalameta.org/metals/docs/integrations/new-editor#metals-http-client) (running on http://localhost:5031 by default)
for executing client commands, which currently are not supported by zed directly, you can use:

``` json
{
  "lsp": {
    "metals": {
      "binary": {
        "arguments": [
          "-Dmetals.http=on"
        ]
      },
      "initialization_options": {
        "isHttpEnabled": true
      }
    }
  }
}
```
