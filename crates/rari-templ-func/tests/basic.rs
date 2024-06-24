use rari_templ_func::rari_f;
use rari_types::{Arg, ArgError, Quotes, RariEnv};

#[test]
fn basic() {
    #[rari_f]
    fn something(a: String) -> Result<String, ArgError> {
        Ok(format!("some {}", a))
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
