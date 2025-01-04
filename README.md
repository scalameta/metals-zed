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

## Running Tests
The extension supports detecting tests by checking if the test class inherits from specific traits

For ScalaTest:
- AnyWordSpec / WordSpec
- AnyFunSpec / FunSpec
- AnyFunSuite / FunSuite
- AnyFlatSpec / FlatSpec
- AnyFeatureSpec / FeatureSpec
- AnyPropSpec / PropSpec
- AnyFreeSpec / FreeSpec

For Specs2:
- Specification

For Munit:
- FunSuite

For Munit Extensions:
- ScalaCheckSuite
- CatsEffectSuite
- ZSuite
- Http4sSuite
- SnapshotSuite
- RequestResponsePactForger
- HedgehogSuite
- TapirGoldenOpenAPISuite
- TapirGoldenOpenAPIValidatorSuite

For Weaver Test:
- SimpleIOSuite / IOSuite

For ZIO Test:
- ZIOSpecDefault


In order to get the run icon in the gutter, you need to provide zed with a task that run the test class.

The task must have the tag `scala-test` in order to be detected by the editor.

Following is an example task that you can add to your editor, to know more about tasks refer to [Zed Documentation](https://zed.dev/docs/tasks).
```json
[
  {
    "label": "run tests with bloop",
    "name": "run-bloop-test",
    "command": "./run-bloop-tests.sh ${ZED_WORKTREE_ROOT} ${ZED_SYMBOL}",
    "working_directory": "${workspace_root}",
    "environment": {
      "PATH": "${env.PATH}"
    },
    "keystroke": "ctrl-shift-r",
    "tags": ["scala-test"]
  }
]
```

The corresponding **minimal** `run-bloop-test.sh` which must be placed in your project root.
If you want to place it anywhere else you'll have to adjust the task accordingly.
```bash
#!/bin/bash

project_name=$(basename $1) # Extract the project name from the $ZED_WORKTREE_ROOT
filename=$2

bloop test $project_name -o "*$filename*"
```

## Running a main class
In order to get the run icon in the gutter, you need to provide zed with a task that run the main class.

The task must have the tag `scala-main` in order to be detected by the editor.

Following is an example task that you can add to your editor, to know more about tasks refer to [Zed Documentation](https://zed.dev/docs/tasks).
```json
[
  {
    "label": "run main with bloop",
    "name": "run-main",
    "command": "./run-bloop-main.sh ${ZED_WORKTREE_ROOT} ${ZED_FILE} ${ZED_SYMBOL}",
    "working_directory": "${workspace_root}",
    "environment": {
      "PATH": "${env.PATH}"
    },
    "tags": ["scala-main"]
  }
]
```

The corresponding **minimal** `run-bloop-main.sh` which must be placed in your project root.
If you want to place it anywhere else you'll have to adjust the task accordingly.

```json
#!/bin/bash

project_name=$(basename $1) # Extract the project name from the $ZED_WORKTREE_ROOT
filepath=$2
mainname=$3

package_name=$(cat $filepath | grep -E '^package ' | sed 's/package //') # Extract the package name from the main file

bloop run $project_name -m "$package_name.$mainname"
```

## Releasing the extension
To release the extension, you need to bump the version in `Cargo.toml` and `extension.toml` in the root of the repository and create a tag for the version (example bump: [e8b826cb3fc0f5f054aa0012e17824f8904a73f5](https://github.com/scalameta/metals-zed/commit/e8b826cb3fc0f5f054aa0012e17824f8904a73f5).

After that, you need to open up a PR on [zed-industries/extensions](https://github.com/zed-industries/extensions) (example: [zed-industries/extensions#29](https://github.com/zed-industries/extensions/pull/1609/files), the PR must :
- Update the submodule on extensions/scala to the appropriate commit
- Update the file `extensions.toml` to set the appropriate version for the extension in the `[scala]` section
