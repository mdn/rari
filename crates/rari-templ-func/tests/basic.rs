use rari_templ_func::rari_f;
use rari_types::{Arg, ArgError, Quotes, RariEnv};

#[test]
fn basic() {
    #[rari_f]
    fn something(a: String) -> Result<String, ArgError> {
        Ok(format!("some {a}"))
    }

    #[rari_f]
    fn something_else(a: Option<String>) -> Result<String, ArgError> {
        Ok(format!("else {}", a.unwrap_or_default()))
    }

    #[rari_f]
    fn many(a: i64, b: Option<i64>) -> Result<String, ArgError> {
        Ok(format!("{} {}", a, b.unwrap_or_default()))
    }

    #[rari_f]
    fn booly(b: Option<bool>) -> Result<String, ArgError> {
        Ok(format!("{}", b.unwrap_or_default()))
    }

    assert_eq!(
        something(&Default::default(), "foo".into()).unwrap(),
        "some foo"
    );
    assert_eq!(
        something_any(
            &Default::default(),
            vec![Some(Arg::String("foo".into(), Quotes::Double))]
        )
        .unwrap(),
        "some foo"
    );
    assert_eq!(
        many_any(
            &Default::default(),
            vec![Some(Arg::Int(1)), Some(Arg::Int(2))]
        )
        .unwrap(),
        "1 2"
    );
    assert_eq!(
        many_any(&Default::default(), vec![Some(Arg::Int(1))]).unwrap(),
        "1 0"
    );

    assert_eq!(booly_any(&Default::default(), vec![]).unwrap(), "false");
}

#[test]
fn arg_errors_name_the_offending_argument() {
    #[rari_f]
    fn link(url: String, display: Option<String>) -> Result<String, ArgError> {
        Ok(format!("{url} {}", display.unwrap_or_default()))
    }

    let blank = || Some(Arg::String(String::new(), Quotes::Double));
    let cases: Vec<(&str, Vec<Option<Arg>>, &str)> = vec![
        (
            "missing required argument",
            vec![],
            "link argument 1 (url) must be provided",
        ),
        (
            "unparseable required argument",
            vec![None],
            "link argument 1 (url) could not be parsed",
        ),
        (
            "blank required argument",
            vec![blank()],
            "link argument 1 (url) must not be empty",
        ),
        (
            "ill-typed required argument",
            vec![Some(Arg::Int(1))],
            "link argument 1 (url) must be a string",
        ),
        (
            "ill-typed optional argument",
            vec![
                Some(Arg::String("a".into(), Quotes::Double)),
                Some(Arg::Int(1)),
            ],
            "link argument 2 (display) must be a string",
        ),
    ];
    for (name, args, expected) in cases {
        let e = link_any(&Default::default(), args).unwrap_err();
        assert_eq!(e.to_string(), expected, "{name}");
    }

    // Blank optional arguments count as absent.
    assert_eq!(
        link_any(
            &Default::default(),
            vec![Some(Arg::String("a".into(), Quotes::Double)), blank()]
        )
        .unwrap(),
        "a "
    );
}

#[test]
fn env() {
    #[rari_f]
    fn something(a: String) -> Result<String, ArgError> {
        Ok(format!("some {}{}", env.title, a))
    }
    assert_eq!(
        something(
            &RariEnv {
                title: "foo",
                ..Default::default()
            },
            "bar".into()
        )
        .unwrap(),
        "some foobar"
    );
}
