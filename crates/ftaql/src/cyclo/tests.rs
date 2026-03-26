#[cfg(test)]
mod tests {
    use crate::cyclo::cyclo;
    use crate::parse::parse_module_from_string;

    fn get_cyclo(source_code: &str) -> usize {
        let (module_res, _) = parse_module_from_string(source_code, true, false);
        let module = module_res.unwrap();
        cyclo(&module)
    }

    #[test]
    fn test_empty_module() {
        let ts_code = r#"
            /* Empty TypeScript code */
        "#;
        assert_eq!(get_cyclo(ts_code), 1);
    }

    #[test]
    fn test_single_if() {
        let source_code = "if (a) {}";
        assert_eq!(get_cyclo(source_code), 2);
    }

    #[test]
    fn test_if_else() {
        let source_code = "if (a) {} else {}";
        assert_eq!(get_cyclo(source_code), 2);
    }

    #[test]
    fn test_nested_ifs() {
        let ts_code = r#"
            if (x > 0) {
                if (x < 10) {
                    console.log("x is between 0 and 10");
                }
            } else {
                console.log("x is not positive");
            }
        "#;
        assert_eq!(get_cyclo(ts_code), 3);
    }

    #[test]
    fn test_switch_case() {
        let ts_code = r#"
            switch (x) {
                case 0:
                    console.log("x is 0");
                    break;
                case 1:
                    console.log("x is 1");
                    break;
                default:
                    console.log("x is not 0 or 1");
            }
        "#;
        assert_eq!(get_cyclo(ts_code), 4);
    }

    #[test]
    fn test_for_loop() {
        let ts_code = r#"
            for (let i = 0; i < 10; i++) {
                console.log(i);
            }
        "#;
        assert_eq!(get_cyclo(ts_code), 2);
    }

    #[test]
    fn test_while_loop() {
        let ts_code = r#"
        let i = 0;
        while (i < 10) {
            console.log(i);
            i++;
        }
    "#;
        assert_eq!(get_cyclo(ts_code), 2);
    }

    #[test]
    fn test_do_while_loop() {
        let ts_code = r#"
        let i = 0;
        do {
            console.log(i);
            i++;
        } while (i < 10);
    "#;
        assert_eq!(get_cyclo(ts_code), 2);
    }

    #[test]
    fn test_for_in_loop() {
        let ts_code = r#"
        let obj = { a: 1, b: 2, c: 3 };
        for (let key in obj) {
            console.log(key, obj[key]);
        }
    "#;
        assert_eq!(get_cyclo(ts_code), 2);
    }

    #[test]
    fn test_for_of_loop() {
        let ts_code = r#"
        let arr = [1, 2, 3];
        for (let item of arr) {
            console.log(item);
        }
    "#;
        assert_eq!(get_cyclo(ts_code), 2);
    }

    #[test]
    fn test_try_catch() {
        let ts_code = r#"
        try {
            throw new Error("An error occurred");
        } catch (e) {
            console.log(e.message);
        }
    "#;
        assert_eq!(get_cyclo(ts_code), 2);
    }

    #[test]
    fn test_conditional_expression() {
        let ts_code = r#"
        let result = x > 0 ? "positive" : "non-positive";
    "#;
        assert_eq!(get_cyclo(ts_code), 2);
    }

    #[test]
    fn comments_have_no_impact_on_complexity() {
        let uncommented_code = r##"
        let obj = {
            ['computed' + 'Property']: 'value'
        };
        class MyClass {
            [Symbol.iterator]() {}
        }
        class MyClassTwo {
            #privateField = 'value';
            getPrivateField() {
                return this.#privateField;
            }
        }
      "##;
        let commented_code = r##"
        // Define an object with a computed property
        let obj = {
            ['computed' + 'Property']: 'value'
        };
        class MyClass {
            [Symbol.iterator]() {}
        }
        class MyClassTwo {
            #privateField = 'value';
            getPrivateField() {
                return this.#privateField;
            }
        }
      "##;
        assert_eq!(get_cyclo(uncommented_code), get_cyclo(commented_code));
    }
}
