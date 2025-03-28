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

;Common Runnables for Cats Effect
(
    (
        (object_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_app_object_end
        (#match? @run "^IOApp?\.Simple")
    )
    (#set! tag scala-main)
)

;Common Runnables for ZIO
(
    (
        (object_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_app_object_end
        (#match? @run "^ZIOApp?(Default)")
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

; ScalaTest Common Runnables - https://www.scalatest.org/
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

; Munit Common Runnables - outside of FunSuite all other keywords were derived from links found in https://scalameta.org/munit/docs/integrations/external-integrations.html
(
    (
        (class_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_test_class_end
        (#match? @run "^(munit\\.)?(FunSuite|ScalaCheckSuite|CatsEffectSuite|Http4sSuite|((snapshot\\.)?SnapshotSuite)|ZSuite|RequestResponsePactForger|HedgehogSuite|TapirGoldenOpenAPISuite|TapirGoldenOpenAPIValidatorSuite)$")
    )
    (#set! tag scala-test)
)

; Specs2 Common Runnables - https://etorreborre.github.io/specs2/guide/SPECS2-5.5.8/org.specs2.guide.UserGuide.html
(
    (
        (class_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_test_class_end
        (#match? @run "^((specs2\\.)?(mutable\\.)?)?(Specification|SpecificationLike|Spec|SpecLike)")
    )
    (#set! tag scala-test)
)

; Weaver Test - https://disneystreaming.github.io/weaver-test/
(
    (
        (class_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_test_class_end
        (#match? @run "^((Simple)?IOSuite)$")
    )
    (#set! tag scala-test)
)

; ZIO Test - https://zio.dev/reference/test/
(
    (
        (class_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_test_class_end
        (#match? @run "^((test\\.)?ZIOSpecDefault)$")
    )
    (#set! tag scala-test)
)

; Hedgehog - https://hedgehogqa.github.io/scala-hedgehog/
(
    (
        (class_definition
            extend: (extends_clause
                type: (type_identifier) @run
            )
        ) @_scala_test_class_end
        (#match? @run "^Properties$")
    )
    (#set! tag scala-test)
)
