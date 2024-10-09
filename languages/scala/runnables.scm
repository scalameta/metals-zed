(
    (
        (function_definition
            (annotation
                name: (type_identifier) @run
            )
            name: _
            parameters: _
            body: _
        ) @_scala_main_function_end
        (#eq? @run "main")
    )
    (#set! tag scala-main)
)

(
    (
        (object_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_app_object_end
        (#eq? @run "App")
    )
    (#set! tag scala-main)
)
