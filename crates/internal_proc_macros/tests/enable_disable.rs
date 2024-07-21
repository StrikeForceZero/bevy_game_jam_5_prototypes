#[cfg(test)]
mod tests {
    use internal_proc_macros::EnableDisable;
    use internal_shared::enable_disable::{EnableDisable, GetEnabledDisabled, SetEnabledDisabled};

    mod multi_named_field {
        use super::*;

        #[derive(Debug, PartialEq, EnableDisable)]
        struct TestStruct {
            #[enable_disable]
            enabled: bool,
            other: u32,
        }

        #[test]
        fn test_enable_disable_impl() {
            fn do_something<T: EnableDisable>(thing: T) -> T {
                thing
            }
            do_something(TestStruct {
                enabled: true,
                other: 0,
            });
        }

        #[test]
        fn test_is_enabled() {
            let test_instance = TestStruct {
                enabled: true,
                other: 0,
            };

            assert!(test_instance.is_enabled());
        }

        #[test]
        fn test_is_disabled() {
            let test_instance = TestStruct {
                enabled: false,
                other: 0,
            };

            assert!(test_instance.is_disabled());
        }

        #[test]
        fn test_set_enabled() {
            let mut test_instance = TestStruct {
                enabled: false,
                other: 0,
            };

            test_instance.set_enabled();
            assert_eq!(
                test_instance,
                TestStruct {
                    enabled: true,
                    other: 0,
                }
            );
        }

        #[test]
        fn test_set_disabled() {
            let mut test_instance = TestStruct {
                enabled: true,
                other: 0,
            };

            test_instance.set_disabled();
            assert_eq!(
                test_instance,
                TestStruct {
                    enabled: false,
                    other: 0,
                }
            );
        }

        #[test]
        fn test_set_enabled_disabled() {
            let mut test_instance = TestStruct {
                enabled: true,
                other: 0,
            };

            test_instance.set_enabled_disabled(false);
            assert_eq!(
                test_instance,
                TestStruct {
                    enabled: false,
                    other: 0,
                }
            );

            test_instance.set_enabled_disabled(true);
            assert_eq!(
                test_instance,
                TestStruct {
                    enabled: true,
                    other: 0,
                }
            );
        }
    }
    mod single_named_field {
        use super::*;

        #[derive(Debug, PartialEq, EnableDisable)]
        struct TestStruct {
            enabled: bool,
        }

        #[test]
        fn test_enable_disable_impl() {
            fn do_something<T: EnableDisable>(thing: T) -> T {
                thing
            }
            do_something(TestStruct { enabled: true });
        }

        #[test]
        fn test_is_enabled() {
            let test_instance = TestStruct { enabled: true };

            assert!(test_instance.is_enabled());
        }

        #[test]
        fn test_is_disabled() {
            let test_instance = TestStruct { enabled: false };

            assert!(test_instance.is_disabled());
        }

        #[test]
        fn test_set_enabled() {
            let mut test_instance = TestStruct { enabled: false };

            test_instance.set_enabled();
            assert_eq!(test_instance, TestStruct { enabled: true });
        }

        #[test]
        fn test_set_disabled() {
            let mut test_instance = TestStruct { enabled: true };

            test_instance.set_disabled();
            assert_eq!(test_instance, TestStruct { enabled: false });
        }

        #[test]
        fn test_set_enabled_disabled() {
            let mut test_instance = TestStruct { enabled: true };

            test_instance.set_enabled_disabled(false);
            assert_eq!(test_instance, TestStruct { enabled: false });

            test_instance.set_enabled_disabled(true);
            assert_eq!(test_instance, TestStruct { enabled: true });
        }
    }
    mod multi_unnamed_field {
        use super::*;

        #[derive(Debug, PartialEq, EnableDisable)]
        struct TestStruct(#[enable_disable] bool, u32);

        #[test]
        fn test_enable_disable_impl() {
            fn do_something<T: EnableDisable>(thing: T) -> T {
                thing
            }
            do_something(TestStruct(true, 0));
        }

        #[test]
        fn test_is_enabled() {
            let test_instance = TestStruct(true, 0);

            assert!(test_instance.is_enabled());
        }

        #[test]
        fn test_is_disabled() {
            let test_instance = TestStruct(false, 0);

            assert!(test_instance.is_disabled());
        }

        #[test]
        fn test_set_enabled() {
            let mut test_instance = TestStruct(false, 0);

            test_instance.set_enabled();
            assert_eq!(test_instance, TestStruct(true, 0));
        }

        #[test]
        fn test_set_disabled() {
            let mut test_instance = TestStruct(true, 0);

            test_instance.set_disabled();
            assert_eq!(test_instance, TestStruct(false, 0));
        }

        #[test]
        fn test_set_enabled_disabled() {
            let mut test_instance = TestStruct(true, 0);

            test_instance.set_enabled_disabled(false);
            assert_eq!(test_instance, TestStruct(false, 0));

            test_instance.set_enabled_disabled(true);
            assert_eq!(test_instance, TestStruct(true, 0));
        }
    }
    mod single_unnamed_field {
        use super::*;

        #[derive(Debug, PartialEq, EnableDisable)]
        struct TestStruct(bool);

        #[test]
        fn test_enable_disable_impl() {
            fn do_something<T: EnableDisable>(thing: T) -> T {
                thing
            }
            do_something(TestStruct(true));
        }

        #[test]
        fn test_is_enabled() {
            let test_instance = TestStruct(true);

            assert!(test_instance.is_enabled());
        }

        #[test]
        fn test_is_disabled() {
            let test_instance = TestStruct(false);

            assert!(test_instance.is_disabled());
        }

        #[test]
        fn test_set_enabled() {
            let mut test_instance = TestStruct(false);

            test_instance.set_enabled();
            assert_eq!(test_instance, TestStruct(true));
        }

        #[test]
        fn test_set_disabled() {
            let mut test_instance = TestStruct(true);

            test_instance.set_disabled();
            assert_eq!(test_instance, TestStruct(false));
        }

        #[test]
        fn test_set_enabled_disabled() {
            let mut test_instance = TestStruct(true);

            test_instance.set_enabled_disabled(false);
            assert_eq!(test_instance, TestStruct(false));

            test_instance.set_enabled_disabled(true);
            assert_eq!(test_instance, TestStruct(true));
        }
    }
    mod nested {
        use super::*;

        #[derive(Debug, PartialEq, EnableDisable)]
        struct TestStruct {
            enabled: TestStruct2,
        }

        #[derive(Debug, PartialEq, EnableDisable)]
        struct TestStruct2 {
            enabled: bool,
        }

        #[test]
        fn test_set_get() {
            let mut instance = TestStruct {
                enabled: TestStruct2 { enabled: false },
            };
            instance.set_enabled();
            assert_eq!(
                instance,
                TestStruct {
                    enabled: TestStruct2 { enabled: true }
                }
            );
        }
    }
}
