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

## Inlay hints

To turn on inlay hints you can use the configuration below:
```json
{
  "inlay_hints": {
    "enabled": true,
    "show_type_hints": true,
    "show_parameter_hints": true,
    "show_other_hints": true,
    "show_background": false,
    "edit_debounce_ms": 700,
    "scroll_debounce_ms": 50
  },
  "lsp": {
    "metals": {
      "settings": {
        "inlayHints": {
          "inferredTypes": {
            "enable": true
          },
          "implicitArguments": {
            "enable": true
          },          
          "implicitConversions": {
            "enable": true
          },          
          "typeParameters": {
            "enable": true
          },          
          "hintsInPatternMatch": {
            "enable": true
          }
        }
      }
    }
  }
}
```

Both sections need to be set, since Zed doesn't turn on inlay hints by default and Metals also needs to know which hints users wants to see.

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
- Specification / SpecificationLike
- Spec / SpecLike

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

For Hedgehog:
- Properties

For Weaver Test:
- SimpleIOSuite / IOSuite

For ZIO Test:
- ZIOSpecDefault

If your favorite test framework is not included, or more traits are have been added, please update the `runnables.scm` file found in the `languages/scala` directory

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

The corresponding **minimal** `run-bloop-tests.sh` which must be placed in your project root.
If you want to place it anywhere else you'll have to adjust the task accordingly. The script must be executable `chmod +x ./run-bloop-tests.sh`.
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

## Debugging (for JVM)

The extension supports debugging through DAP (Debug Adapter Protocol). To debug your Scala code, you typically need to provide a proper debug task definition. Please see [Zed Debugger](https://zed.dev/docs/debugger), and specifically [its configuration](https://zed.dev/docs/debugger#configuration), for general overview. In simple cases, you may spawn the debugger without a definition - see [Generic configuration](#generic-configuration) below for the details.

You may define global and local (per workspace) debug task definitions. The local definitions go into `.zed/debug.json` - you may open or create the file through Zed's menu: `Run/Edit debug.json`. Access to the global ones is described in  [Global debug configurations](https://zed.dev/docs/debugger#global-debug-configurations).
You may define as many debug task definitions as you like - they will be available to select in the `Run/Start Debugger` menu.

JSON schema for the debug task definition may be found [here](debug_adapter_schemas/Metals.json). Keep reading for a less formal description.

Three fields are required for any debug scenario:
```json
  {
    "label": "My debug configuration",  // Your name of the configuration
    "adapter": "Metals",  // Always "Metals"
    "request": "launch" // "launch" or "attach" described below
  }
```
The above fields are ZED-specific. The rest is [Metals-specific](https://scalameta.org/metals/docs/integrations/debug-adapter-protocol#via-explicit-main-or-test-commands). While reading Metals debugging documentation, bear in mind that running and debugging via code lenses is not supported in Zed.

Debugging is possible in two modes, launching a new instance or attaching to a running one, as described in [Launching & Attaching](https://zed.dev/docs/debugger#launching--attaching) and below.

### Launching

For all launch scenarios, the `request` field must be set to `"launch"`.

The simplest and most general way of starting a debug session is through configuration, which uses Metals autodiscovery:
```json
  {
    "label": "Debug Scala - autodiscover",
    "adapter": "Metals",
    "request": "launch",
    "path": "$ZED_FILE",
    "runType": "run"
  }
```
The `path` has to point to a Scala source file in your project that should be run (main or test, depending on `runType`). If a relative path is provided, it will get prefixed with the workspace root. If the file doesn't contain a runnable method, Metals will try to identify one in the current project and build target (see more about build targets below). Please note that the main and test sources form two build targets. This means that autodiscovery won't work for the main class if a test file is indicated and vice versa.

Zed supports [task variables](https://zed.dev/docs/tasks#variables) in debug task definitions. `$ZED_FILE` is especially handy for debugging scenarios - it is replaced with the full path of the currently open file. This makes the above definition quite universal and suitable for global configuration.

Metals support the following `runType`s:
- `"run"` - run the main method in the build target the indicated file belongs to - this is the default if `runTime` is omitted in the configuration,
- `"runOrTestFile"` - run the main or test class in the indicated file,
- `"testFile"` - run test class in the indicated file,
- `"testTarget"` - run all test classes in the build target the indicated file belongs to.

You may provide additional parameters to the configuration:
- `args` - array of arguments to pass to the launched method,
- `jvmOptions` - Java virtual machine options (eg., memory settings),
- `env` - environment variables,
- `envFile` - file containing environment variables.
If both `env` and `envFile` are provided, the `env` definitions take precedence over those in `envFile`. For a detailed description of the environment file format, see [Metals debugging documentation](https://scalameta.org/metals/docs/integrations/debug-adapter-protocol#via-explicit-main-or-test-commands).

A full version of the debug launch configuration, which uses autodiscovery, may look as follows:
```json
  {
    "label": "Debug Scala - autodiscover",
    "adapter": "Metals",
    "request": "launch",
    "path": "/projects/foo/src/test/scala/dev/foo/FooTest.scala",
    "runType": "testFile",
    "args": ["bar","baz"],
    "jvmOptions": ["-Xms1G", "-Xmx4G", "-Dproperty=123"],
    "env": { "API_KEY": "a0b1c2d3e4f5g6h7i8j9k0l1m2n3o4p5q6r7s8t9" },
    "envFile": ".env"
  }
```
  
Instead of relying on Metals autodiscovery, you may provide the main or test class explicitly. The basic configuration in that case has the form
```json
  {
    "label": "Debug Scala main class",
    "adapter": "Metals",
    "request": "launch",
    "mainClass": "dev.foo.Foo"
  }
```
for a main class or
```json
  {
    "label": "Debug Scala test class",
    "adapter": "Metals",
    "request": "launch",
    "testClass": "dev.foo.FooTest"
  }
```
for a test class.
Please note that for the main method in the form
```scala
package foo
object App:
  @main
  def run: Unit = ???
```
the proper way to indicate the main class is `"mainClass" = "foo.run"`.

All the additional parameters mentioned for autodiscovery (`args`, `jvmOptions`, `env` and `envFile`) may be provided to main and test class based configurations.

For larger projects, when the same class name may be found in different modules, an additional parameter, `buildTarget`, may be required. The full version of a debug configuration with the main class may have the following form:
```json
  {
    "label": "Debug Scala main class",
    "adapter": "Metals",
    "request": "launch",
    "mainClass": "dev.foo.Foo",
    "buildTarget": "foo-core_d5c0a6989e",
    "args": ["bar","baz"],
    "jvmOptions": ["-Xms1G", "-Xmx4G", "-Dproperty=123"],
    "env": { "API_KEY": "a0b1c2d3e4f5g6h7i8j9k0l1m2n3o4p5q6r7s8t9" },
    "envFile": ".env"
  }
```
Zed doesn't display the names of `buildTarget`s. To find them, Metals should have an HTTP server enabled (`-Dmetals.http=on` system property in LSP configuration), which allows Metals Doctor view under [localhost:5031/doctor](http://localhost:5031/doctor) or on any following port (`5032`, `5033`, etc.) if `5031` was already taken.

### Attaching

In addition to launching a program, you may attach to an already running one. The running program needs to expose a debug endpoint, which needs to pass special parameters to the JVM. When using `scala-cli`, [all you need is to run or test the application with `--debug` option](https://scala-cli.virtuslab.org/docs/cookbooks/introduction/debugging/):
```shell
scala-cli run foo.scala --debug
```
or
```shell
scala-cli test foo.test.scala --debug
```
The debug port will be displayed while starting the application.

The debug task configuration for attach mode has the following form:
```json
  {
    "label": "Attach to Scala program",
    "adapter": "Metals",
    "request": "attach",
    "hostName": "localhost",
    "port": 5005
  }
```
Both `hostName` and `port` may be omitted if the default values (`"localhost"` and `5005` respectively) should be used.

### Generic configuration

In addition to the main debug menu, Zed also provides generic `Attach` and `Launch` UIs. Please note that in both cases, you need to select the `Metals` debugger in the pull-down.

In the `attach` mode, as Metals don’t support attaching to a program by its process id, selecting any process from the list causes the debugger to attach to the default host and port, as described in the previous section.

For the `launch` mode, the provided program is interpreted as the path used by Metals' autodiscovery, with default runType, as described in the [Launching](#launching) section. If you provide a relative path to the program, it will be added to the working directory to get the absolute path. If a full path is provided, the working directory is ignored. You may set environment variables and provide parameters to the launched program, as the prompt in the launch UI suggests.

### Limitations and known problems

For the debug session to start, make sure the Metals LSP server is up and running. If not, you can get one of the following errors:
- `The Metals LSP server hasn't been started yet for the current workspace ...`
- `-32602 Could not find '' build target `
- `-32600 No build target could be found for the path: ...`

While trying to set a breakpoint in a test method, Zed (as of v0.216.1) throws an error: ``invalid value: integer `-1`, expected u64``.

## Releasing the extension
To release the extension, you need to bump the version in `Cargo.toml` and `extension.toml` in the root of the repository and create a tag for the version (example bump: [e8b826cb3fc0f5f054aa0012e17824f8904a73f5](https://github.com/scalameta/metals-zed/commit/e8b826cb3fc0f5f054aa0012e17824f8904a73f5).

After that, you need to open up a PR on [zed-industries/extensions](https://github.com/zed-industries/extensions) (example: [zed-industries/extensions#29](https://github.com/zed-industries/extensions/pull/1609/files), the PR must :
- Update the submodule on extensions/scala to the appropriate commit
- Update the file `extensions.toml` to set the appropriate version for the extension in the `[scala]` section
