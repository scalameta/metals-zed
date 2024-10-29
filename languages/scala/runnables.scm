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

(
    (
        (object_definition
            name: _
            body: (template_body
                (function_definition
                    name: (identifier) @run
                ) @_jvm_main_function_end
            )
        )
        (#eq? @run "main")
    )
    (#set! tag scala-main)
)

(
    (
        (class_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_test_class_end
        (#match? @run "^(AnyWordSpec|WordSpec|AnyFunSpec|FunSpec|AnyFunSuite|FunSuite|AnyFlatSpec|FlatSpec|FeatureSpec|AnyFeatureSpec|AnyPropSpec|PropSpec|AnyFreeSpec|FreeSpec)$")
    )
    (#set! tag scala-test)
)
